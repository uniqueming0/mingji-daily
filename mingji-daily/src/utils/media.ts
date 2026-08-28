export const PHOTO_EXTS = ["jpg", "jpeg", "png", "gif", "webp", "bmp"];
export const VIDEO_EXTS = ["mp4", "webm", "mov", "m4v"];

export function extOf(path: string): string {
  return (path.split(".").pop() ?? "").toLowerCase();
}

export function isPhoto(path: string): boolean {
  return PHOTO_EXTS.includes(extOf(path));
}

export function isVideo(path: string): boolean {
  return VIDEO_EXTS.includes(extOf(path));
}
