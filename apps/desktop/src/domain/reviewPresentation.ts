export type ReviewPresentationScope = {
  generation: number;
  selectedPath: readonly number[];
  requestToken: string;
};

export function createLocalRequestToken(nextSerial: () => number): string {
  return `local:${nextSerial()}`;
}

export function shouldPublishReviewPresentation(
  active: ReviewPresentationScope,
  captured: ReviewPresentationScope
): boolean {
  return active.generation === captured.generation
    && active.requestToken === captured.requestToken
    && active.selectedPath.length === captured.selectedPath.length
    && active.selectedPath.every((index, offset) => index === captured.selectedPath[offset]);
}
