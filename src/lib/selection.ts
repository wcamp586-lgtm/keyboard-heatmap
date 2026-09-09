export const nextSelectedKeyId = (
  selectedKeyId: number | null,
  clickedKeyId: number,
): number | null => selectedKeyId === clickedKeyId ? null : clickedKeyId
