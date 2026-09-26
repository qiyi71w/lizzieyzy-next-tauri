import type { GameDto, MoveDto, MoveVertex, PlayerColor, PointDto, PositionDto, StoneDto } from "./types";

export type Stone = PointDto & { color: PlayerColor; moveNumber: number };

export function isPoint(vertex: MoveVertex): vertex is { point: PointDto } {
  return typeof vertex === "object" && vertex !== null && "point" in vertex;
}

export function replayMainLine(game: GameDto, untilMove = game.moves.length): Stone[] {
  const stones = new Map<string, Stone>();
  for (const move of game.moves.slice(0, untilMove)) {
    if (!isPoint(move.vertex)) continue;
    const { x, y } = move.vertex.point;
    stones.set(`${x}:${y}`, { x, y, color: move.color, moveNumber: move.move_number });
  }
  return [...stones.values()];
}

export function createInitialPosition(boardWidth: number, boardHeight: number): PositionDto {
  return buildPosition(boardWidth, boardHeight, 0, "black", new Map(), 0, 0, null, []);
}

export function ensureInitialPosition(boardWidth: number, boardHeight: number, positions: PositionDto[]): PositionDto[] {
  if (positions.some((position) => position.move_number === 0)) return positions;
  return [createInitialPosition(boardWidth, boardHeight), ...positions];
}

export function selectExactPosition(positions: PositionDto[], moveNumber: number, boardWidth: number, boardHeight: number): PositionDto {
  return positions.find((position) => position.move_number === moveNumber) ?? createInitialPosition(boardWidth, boardHeight);
}

export function clampMoveNumberToPositions(positions: PositionDto[], moveNumber: number): number {
  const moveNumbers = positions.map((position) => position.move_number).sort((a, b) => a - b);
  if (moveNumbers.length === 0) return 0;
  if (moveNumbers.includes(moveNumber)) return moveNumber;
  return moveNumbers.reduce((best, candidate) => Math.abs(candidate - moveNumber) < Math.abs(best - moveNumber) ? candidate : best);
}

export function isImmediateKoPoint(point: PointDto, koPoint: PointDto | null): boolean {
  return koPoint !== null && point.x === koPoint.x && point.y === koPoint.y;
}

export function nextKoPoint(capturedStones: StoneDto[], ownGroup: StoneDto[]): PointDto | null {
  if (capturedStones.length !== 1 || ownGroup.length !== 1) return null;
  return { x: capturedStones[0].x, y: capturedStones[0].y };
}

export function replayGamePositions(game: GameDto): PositionDto[] {
  const boardWidth = game.summary.board_width;
  const boardHeight = game.summary.board_height;
  const stones = new Map<string, StoneDto>();
  let capturesBlack = 0;
  let capturesWhite = 0;
  let toPlay: PlayerColor = "black";
  let koPoint: PointDto | null = null;
  const positions: PositionDto[] = [createInitialPosition(boardWidth, boardHeight)];

  for (const move of game.moves) {
    const errors: string[] = [];

    if (isPoint(move.vertex)) {
      const { x, y } = move.vertex.point;
      if (!isOnBoard(x, y, boardWidth, boardHeight)) {
        errors.push(`Move ${move.move_number} is outside the ${boardWidth}x${boardHeight} board.`);
      } else if (isImmediateKoPoint(move.vertex.point, koPoint)) {
        errors.push(`Move ${move.move_number} violates simple ko.`);
      } else if (stones.has(pointKey(x, y))) {
        errors.push(`Move ${move.move_number} tried to play on an occupied point.`);
      } else {
        stones.set(pointKey(x, y), { x, y, color: move.color });
        const capturedStones = captureAdjacentOpponentGroups(stones, x, y, move.color, boardWidth, boardHeight);
        const ownGroup = collectGroup(stones, x, y, boardWidth, boardHeight);
        if (ownGroup && countLiberties(stones, ownGroup, boardWidth, boardHeight) === 0) {
          errors.push(`Move ${move.move_number} has no liberties.`);
          stones.delete(pointKey(x, y));
          for (const capturedStone of capturedStones) stones.set(pointKey(capturedStone.x, capturedStone.y), capturedStone);
        } else if (move.color === "black") {
          capturesBlack += capturedStones.length;
          koPoint = ownGroup ? nextKoPoint(capturedStones, ownGroup) : null;
        } else {
          capturesWhite += capturedStones.length;
          koPoint = ownGroup ? nextKoPoint(capturedStones, ownGroup) : null;
        }
      }
    } else {
      koPoint = null;
    }

    toPlay = move.color === "black" ? "white" : "black";
    positions.push(buildPosition(boardWidth, boardHeight, move.move_number, toPlay, stones, capturesBlack, capturesWhite, move, errors));
  }

  return positions;
}

export function vertexLabel(vertex: MoveVertex, boardHeight: number): string {
  if (!isPoint(vertex)) return "pass";
  const letters = "ABCDEFGHJKLMNOPQRSTUVWXYZ";
  const col = letters[vertex.point.x] ?? "?";
  return `${col}${boardHeight - vertex.point.y}`;
}

function buildPosition(
  boardWidth: number,
  boardHeight: number,
  moveNumber: number,
  toPlay: PlayerColor,
  stones: Map<string, StoneDto>,
  capturesBlack: number,
  capturesWhite: number,
  lastMove: MoveDto | null,
  errors: string[]
): PositionDto {
  return {
    board_width: boardWidth,
    board_height: boardHeight,
    move_number: moveNumber,
    to_play: toPlay,
    stones: [...stones.values()],
    captures_black: capturesBlack,
    captures_white: capturesWhite,
    last_move: lastMove,
    errors
  };
}

function captureAdjacentOpponentGroups(stones: Map<string, StoneDto>, x: number, y: number, color: PlayerColor, boardWidth: number, boardHeight: number): StoneDto[] {
  const opponent = color === "black" ? "white" : "black";
  const captured: StoneDto[] = [];
  const checked = new Set<string>();
  for (const [nx, ny] of neighbors(x, y, boardWidth, boardHeight)) {
    const neighbor = stones.get(pointKey(nx, ny));
    if (!neighbor || neighbor.color !== opponent || checked.has(pointKey(nx, ny))) continue;
    const group = collectGroup(stones, nx, ny, boardWidth, boardHeight);
    if (!group) continue;
    for (const stone of group) checked.add(pointKey(stone.x, stone.y));
    if (countLiberties(stones, group, boardWidth, boardHeight) > 0) continue;
    for (const stone of group) {
      stones.delete(pointKey(stone.x, stone.y));
      captured.push(stone);
    }
  }
  return captured;
}

function collectGroup(stones: Map<string, StoneDto>, x: number, y: number, boardWidth: number, boardHeight: number): StoneDto[] | null {
  const start = stones.get(pointKey(x, y));
  if (!start) return null;
  const group: StoneDto[] = [];
  const visited = new Set<string>();
  const queue = [start];
  while (queue.length > 0) {
    const stone = queue.shift();
    if (!stone) continue;
    const key = pointKey(stone.x, stone.y);
    if (visited.has(key)) continue;
    visited.add(key);
    group.push(stone);
    for (const [nx, ny] of neighbors(stone.x, stone.y, boardWidth, boardHeight)) {
      const neighbor = stones.get(pointKey(nx, ny));
      if (neighbor?.color === start.color) queue.push(neighbor);
    }
  }
  return group;
}

function countLiberties(stones: Map<string, StoneDto>, group: StoneDto[], boardWidth: number, boardHeight: number): number {
  const liberties = new Set<string>();
  for (const stone of group) {
    for (const [nx, ny] of neighbors(stone.x, stone.y, boardWidth, boardHeight)) {
      if (!stones.has(pointKey(nx, ny))) liberties.add(pointKey(nx, ny));
    }
  }
  return liberties.size;
}

function neighbors(x: number, y: number, boardWidth: number, boardHeight: number): Array<[number, number]> {
  return [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]]
    .filter(([nx, ny]) => isOnBoard(nx, ny, boardWidth, boardHeight)) as Array<[number, number]>;
}

function isOnBoard(x: number, y: number, boardWidth: number, boardHeight: number): boolean {
  return x >= 0 && y >= 0 && x < boardWidth && y < boardHeight;
}

function pointKey(x: number, y: number): string {
  return `${x}:${y}`;
}

export function createDemoGame(): GameDto {
  const moves: MoveDto[] = [
    { color: "black", vertex: { point: { x: 15, y: 3 } }, move_number: 1 },
    { color: "white", vertex: { point: { x: 3, y: 15 } }, move_number: 2 },
    { color: "black", vertex: { point: { x: 15, y: 15 } }, move_number: 3 },
    { color: "white", vertex: { point: { x: 3, y: 3 } }, move_number: 4 },
    { color: "black", vertex: { point: { x: 10, y: 16 } }, move_number: 5 },
    { color: "white", vertex: { point: { x: 16, y: 10 } }, move_number: 6 }
  ];
  return { summary: { id: "demo", board_width: 19, board_height: 19, komi: 7.5, black_name: "李昌镐", white_name: "芮乃伟", result: null, move_count: moves.length }, moves };
}
