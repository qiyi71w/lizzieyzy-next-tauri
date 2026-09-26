import { describe, expect, it } from "vitest";
import { newDocumentSgf } from "./newDocument";

describe("newDocumentSgf", () => {
  it("encodes square and rectangular dimensions and escapes names", () => {
    expect(newDocumentSgf({
      boardWidth: 19,
      boardHeight: 19,
      komi: 7.5,
      blackName: "黑",
      whiteName: "白"
    })).toContain("SZ[19]");

    expect(newDocumentSgf({
      boardWidth: 25,
      boardHeight: 2,
      komi: -0.5,
      blackName: "黑]方",
      whiteName: "白\\方\n二"
    })).toBe("(;GM[1]FF[4]SZ[25:2]KM[-0.5]PB[黑\\]方]PW[白\\\\方 二])");
  });
});
