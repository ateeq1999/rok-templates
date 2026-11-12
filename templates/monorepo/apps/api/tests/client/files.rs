//! Files through the client: an upload's bytes in, its metadata and bytes
//! out, and the profile photo — what `media.rs` and `avatars.rs` drive
//! through the router.

use std::io::Cursor;

use api_client::{MediaListQuery, MediaUploadQuery};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};

use crate::support::{TestResult, user};
use crate::{Api, Error};

/// A PNG's first bytes: enough for type detection.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0";

/// A 400×300 PNG.
fn picture() -> Result<Vec<u8>, Error> {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(400, 300, Rgba([30, 90, 200, 255])));
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    Ok(bytes)
}

#[tokio::test]
async fn a_file_is_uploaded_listed_read_and_removed() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    api.client.set_token(Some(user(&api.db.pool).await?));
    let named = MediaUploadQuery {
        filename: Some("dot.png".to_owned()),
    };
    let created = api.client.media().upload(PNG.to_vec(), &named).await?;
    assert_eq!(created.content_type, "image/png");
    assert_eq!(created.size_bytes, i64::try_from(PNG.len())?);
    assert_eq!(created.original_filename.as_deref(), Some("dot.png"));

    assert_eq!(api.client.media().metadata(&created.id).await?, created);
    assert_eq!(api.client.media().content(&created.id).await?, PNG);
    let page = api.client.media().list(&MediaListQuery::default()).await?;
    assert_eq!(page.total, 1);
    assert_eq!(page.items, vec![created.clone()]);

    api.client.media().remove(&created.id).await?;
    let err = api
        .client
        .media()
        .metadata(&created.id)
        .await
        .err()
        .ok_or("a removed file was found")?;
    assert_eq!((err.status, err.code.as_str()), (404, "not_found"));
    api.stop().await
}

#[tokio::test]
async fn a_profile_photo_is_set_served_and_cleared() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    api.client.set_token(Some(user(&api.db.pool).await?));
    // No photo yet is `None`, not an error.
    assert_eq!(api.client.avatars().current().await?, None);
    let me = api.client.avatars().upload(picture()?).await?;
    let url = me.avatar_url.ok_or("no avatar_url")?;
    let current = api.client.avatars().current().await?;
    assert_eq!(
        current.map(|avatar| avatar.url).as_deref(),
        Some(url.as_str())
    );
    let file = url
        .strip_prefix("/api/v1/avatars/")
        .ok_or(format!("not an avatar's URL: {url}"))?;
    let served = api.client.avatars().serve(file).await?;
    let decoded = image::load_from_memory_with_format(&served, ImageFormat::Jpeg)?;
    assert_eq!((decoded.width(), decoded.height()), (256, 256));

    let me = api.client.avatars().remove().await?;
    assert_eq!(me.avatar_url, None);
    assert_eq!(api.client.users().me().await?.avatar_url, None);
    assert_eq!(api.client.avatars().current().await?, None);
    api.stop().await
}
