import { describe, expect, it } from "vitest";
import { classifyAutoMoveRank, classifyPlayedMove, isBlunderBarRank } from "./moveRank";

describe("Java Auto Move Rank", () => {
  it("uses inclusive winrate-only boundaries", () => {
    expect(classifyAutoMoveRank(0.999)).toBe("best");
    expect(classifyAutoMoveRank(1)).toBe("good");
    expect(classifyAutoMoveRank(3)).toBe("normal");
    expect(classifyAutoMoveRank(6)).toBe("inaccuracy");
    expect(classifyAutoMoveRank(12)).toBe("mistake");
    expect(classifyAutoMoveRank(24)).toBe("blunder");
  });

  it("uses score for ordinary Auto ranks and winrate as Mistake/Blunder guard", () => {
    expect(classifyAutoMoveRank(6, 0.4)).toBe("best");
    expect(classifyAutoMoveRank(0, 0.5)).toBe("good");
    expect(classifyAutoMoveRank(20, 0.1)).toBe("mistake");
    expect(classifyAutoMoveRank(24)).toBe("blunder");
  });

  it("computes same-player loss from parent side-to-play", () => {
    expect(classifyPlayedMove(
      { visits: 10, winrateBlack: 0.6 },
      { visits: 10, winrateBlack: 0.48 },
      "black"
    )).toBe("mistake");
    expect(classifyPlayedMove(
      { visits: 10, winrateBlack: 0.4 },
      { visits: 10, winrateBlack: 0.52 },
      "white"
    )).toBe("mistake");
    expect(classifyPlayedMove(
      { visits: 10, winrateBlack: 0.4 },
      { visits: 10, winrateBlack: 0.52 },
      "black"
    )).toBe("best");
  });

  it("leaves missing, malformed, and zero-visit pairs ungraded", () => {
    const ok = { visits: 8, winrateBlack: 0.55, scoreMeanBlack: 1 };
    expect(classifyPlayedMove(null, ok, "black")).toBeNull();
    expect(classifyPlayedMove(ok, { visits: 0, winrateBlack: 0.1, scoreMeanBlack: -12 }, "black")).toBeNull();
  });

  it("filters Blunder Bar to the last three ranks", () => {
    expect(isBlunderBarRank("best")).toBe(false);
    expect(isBlunderBarRank("normal")).toBe(false);
    expect(isBlunderBarRank("inaccuracy")).toBe(true);
    expect(isBlunderBarRank("blunder")).toBe(true);
  });
});
