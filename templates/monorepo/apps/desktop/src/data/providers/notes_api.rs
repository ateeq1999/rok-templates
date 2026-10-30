//! `NotesApi`: the `notes` API. Every path, verb and wire name lives in this file, so a backend change is a one-place fix.

use crate::data::models::note::NoteId;
use rok_ui::http::path_segment;
use rok_ui::http::ApiError;
use rok_ui::http::Bytes;
use rok_ui::http::HttpClient;
use rok_ui::http::Options;

/// The `notes` endpoints, over the app's [`HttpClient`].
#[derive(Clone, Debug)]
pub struct NotesApi {
    client: HttpClient,
}

impl NotesApi {
    /// The endpoints, called through `client`.
    #[must_use]
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    /// The client (and through it, the session).
    #[must_use]
    pub fn client(&self) -> &HttpClient {
        &self.client
    }

    /// `GET /api/notes`.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn list_notes(&self, search: Option<String>) -> Result<Vec<NoteDto>, ApiError> {
        self.client
            .request("/api/notes", Options::get().query("q", search))
            .await
    }

    /// `POST /api/notes`.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn create_note(&self, body: NewNoteDto) -> Result<NoteDto, ApiError> {
        self.client
            .request("/api/notes", Options::post().json(&body))
            .await
    }

    /// `DELETE /api/notes/{id}`.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn delete_note(&self, id: NoteId) -> Result<(), ApiError> {
        self.client
            .request_empty(
                &format!("/api/notes/{}", path_segment(&id)),
                Options::delete(),
            )
            .await
    }

    /// `PUT /api/notes/{id}/attachment`.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn upload_attachment(
        &self,
        id: NoteId,
        body: Vec<u8>,
        content_type: String,
    ) -> Result<AttachmentDto, ApiError> {
        self.client
            .request(
                &format!("/api/notes/{}/attachment", path_segment(&id)),
                Options::put().raw_body(body).content_type(content_type),
            )
            .await
    }

    /// `GET /api/notes/{id}/attachment`.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn download_attachment(&self, id: NoteId) -> Result<Bytes, ApiError> {
        self.client
            .request_bytes(
                &format!("/api/notes/{}/attachment", path_segment(&id)),
                Options::get(),
            )
            .await
    }
}

/// Note, as the API sends and receives it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoteDto {
    /// `id`.
    pub id: NoteId,
    /// `title`.
    pub title: String,
    /// `body`.
    pub body: String,
    /// `created_at`.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// `attachments`.
    pub attachments: Vec<AttachmentDto>,
}

/// New note, as the API sends and receives it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NewNoteDto {
    /// `title`.
    pub title: String,
    /// `body`.
    pub body: String,
}

/// Attachment, as the API sends it: one of these shapes.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AttachmentDto {
    /// Image.
    Image {
        /// `url`.
        url: String,
        /// `width`.
        width: u32,
    },
    /// File.
    File {
        /// `url`.
        url: String,
        /// `file_name`.
        #[serde(rename = "fileName")]
        file_name: String,
    },
}
