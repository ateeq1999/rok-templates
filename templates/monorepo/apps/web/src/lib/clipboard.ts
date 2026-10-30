/** Copies text; resolves false (instead of throwing) when the browser refuses. */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}

/** Saves text as a file through a temporary object URL. */
export function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob)
  const link = document.createElement("a")
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  // give the browser a tick to start the download before releasing the URL
  setTimeout(() => URL.revokeObjectURL(url), 1000)
}
