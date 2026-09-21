export type NewDocumentParameters = {
  boardWidth: number;
  boardHeight: number;
  komi: number;
  blackName: string;
  whiteName: string;
};
export function escapeSgfValue(value: string): string {
  return value.replace(/\\/g, "\\\\").replace(/\]/g, "\\]").replace(/\r?\n/g, " ");
}


export function newDocumentSgf(parameters: NewDocumentParameters): string {
  const size = parameters.boardWidth === parameters.boardHeight
    ? String(parameters.boardWidth)
    : `${parameters.boardWidth}:${parameters.boardHeight}`;
  return `(;GM[1]FF[4]SZ[${size}]KM[${parameters.komi}]PB[${escapeSgfValue(parameters.blackName)}]PW[${escapeSgfValue(parameters.whiteName)}])`;
}
