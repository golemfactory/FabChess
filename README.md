# FabChess for gWASM
A UCI-compliant chess engine in Rust, ported to gWASM.
Based on the [original engine](https://github.com/fabianvdW/FabChess) written by Fabian von der Warth.

This is only an engine, use [golemate](https://github.com/golemfactory/golemate) to use with Golem Network.

## Compilation instructions
First follow the [gWASM docs] to setup the Emscripten WASM toolchain.

Then execute
```
./build.sh --release
```
to compile the engine in release mode or
```
./build.sh
```
to compile the engine in debug mode.

[gWASM docs]: https://docs.golem.network/#/Products/gWASM/Quick-start

## Known issues
* Running under sp-wasm with high hash sizes may cause crashes. See #1.
* Some of the UCI messages return invalid information, for instance time elapsed is always reported as 0 or used hash is never updated. This is probably not important, though.
* The release build may sometimes take a very long time to instantiate. See #2.

## Wiki
Interested in how it works?

Check out the wiki at: https://github.com/fabianvdW/FabChess/wiki

## Playing strength
| Version       | 40/4    |  40/40 | Comment |
|---------------|---------|--------|---------|
| Current Dev   |     ~   |    ~   |         |
| Version 1.14  |3017/2924|  2906  | See CCRL|
| Version 1.13  |2955-4CPU|  2840  | See CCRL|
| Version 1.12.6| 2788    |  2762  | See CCRL|
| Version 1.12  | 2785    |  -     | See CCRL|
| Version 1.11  |  -      |  2606  | See CCRL|
| Version 1.10  | >2600   |        | Estimate|
| Version 1.9.1 | 2510    |  2442  | See CCRL|
| Version 1.9   | 2534    |  2463  | See CCRL|
| Version 1.8   | 2409    |   -    | See CCRL|
## Usage
FabChess supports more commands than the standard UCI specifies.

### Static evaluation
Use `static` to get a static evaluation of the position
```
> position startpos
> static
< cp 10
```
### Display evaluation
If you compile FabChess with an extra flag, it will also write a detailed overview of the evaluation to a logfile.

!!! Make sure not to run any `go` command with this, else it will quite literally produce a lot of text !!!
```
> cargo run --features "display-eval"
> position startpos
> static
< cp 10
```
Logfile called `log.txt`:
```
Evaluating GameState fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

Tempo:(10 , 15)

PSQT for White:
	Pawns  : (-225 , -216)
	Knights: (-67 , -93)
	Bishops: (-23 , -35)
	Rooks: (24 , -56)
	Queens: (22 , -5)
	King   : (47 , -88)
Sum: (-222 , -493)

PSQT for Black:
	Pawns  : (-225 , -216)
	Knights: (-67 , -93)
	Bishops: (-23 , -35)
	Rooks: (24 , -56)
	Queens: (22 , -5)
	King   : (47 , -88)
Sum: (-222 , -493)

PSQT Sum: (0 , 0)

Piece values for White
	Pawns: 8 -> (888 , 1472)
	Knights: 2 -> (1018 , 1588)
	Bishops: 2 -> (992 , 1422)
	Bishop-Pair: 1 -> (41 , 111)
	Rooks: 2 -> (1310 , 2596)
	Queens: 1 -> (1541 , 2449)
Sum: (5790 , 9638)

Piece values for Black
	Pawns: 8 -> (888 , 1472)
	Knights: 2 -> (1018 , 1588)
	Bishops: 2 -> (992 , 1422)
	Bishop-Pair: 1 -> (41 , 111)
	Rooks: 2 -> (1310 , 2596)
	Queens: 1 -> (1541 , 2449)
Sum: (5790 , 9638)

Piece value Sum: (5790 , 9638) - (5790 , 9638) -> (0 , 0)

Pawns for White:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (270 , 360)
	Passer Blocked/Not Blocked: 0 , 0 -> (0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
	Weak passer: 0 -> (0 , 0)
	Passers distance to kings -> (0 , 0)
Sum: (270 , 360)

Pawns for Black:
	Doubled: 0 -> (0 , 0)
	Isolated: 0 -> (0 , 0)
	Backward: 0 -> (0 , 0)
	Supported: 0 -> (0 , 0)
	Attack Center: 0 -> (0 , 0)
	Mobility: 30 -> (270 , 360)
	Passer Blocked/Not Blocked: 0 , 0 -> (0 , 0)
	Rook behind passer: 0 -> (0 , 0)
	Enemy Rook behind passer: 0 -> (0 , 0)
	Weak passer: 0 -> (0 , 0)
	Passers distance to kings -> (0 , 0)
Sum: (270 , 360)

Pawn Sum: (270 , 360) - (270 , 360) -> (0 , 0)

Knights for White:
	Supported by pawns: 0 -> (0 , 0)
	Outposts: 0 -> (0 , 0)
Sum: (0 , 0)

Knights for Black:
	Supported by pawns: 0 -> (0 , 0)
	Outposts: 0 -> (0 , 0)
Sum: (0 , 0)

Knights Sum: (0 , 0) - (0 , 0) -> (0 , 0)

Piecewise for White:
	Mobility Knight: (-10 , 22)
	Mobility Bishop: (-6 , -62)
	Bishop Diagonally Adj: (-54 , 106)
	Mobility Rook  : (-48 , -52)
	Mobility Queen : (-12 , -40)
	BishopXrayKing : 0 -> (0 , 0)
	RookXrayKing : 0 -> (0 , 0)
	QueenXrayKing : 0 -> (0 , 0)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on semi-open  : 0 -> (0 , 0)
	Queens on open  : 0 -> (0 , 0)
	Queens on semi-open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers: Num: 0 , Val: (0 , 0)
	Bishop Attackers: Num: 0 , Val: (0 , 0)
	Rook Attackers: Num: 0 , Val: (0 , 0)
	Queen Attackers: Num: 0 , Val: (0 , 0)
	Sum Attackers: (Num: 0 , Val: (0 , 0)
	Attack MG value: 0 * 0 / 100.0 -> 0
	Attack EG value: 0 * -1 / 100.0 -> 0
Sum: (-130 , -26)

Piecewise for Black:
	Mobility Knight: (-10 , 22)
	Mobility Bishop: (-6 , -62)
	Bishop Diagonally Adj: (-54 , 106)
	Mobility Rook  : (-48 , -52)
	Mobility Queen : (-12 , -40)
	BishopXrayKing : 0 -> (0 , 0)
	RookXrayKing : 0 -> (0 , 0)
	QueenXrayKing : 0 -> (0 , 0)
	Rooks on open  : 0 -> (0 , 0)
	Rooks on semi-open  : 0 -> (0 , 0)
	Queens on open  : 0 -> (0 , 0)
	Queens on semi-open  : 0 -> (0 , 0)
	Rooks on seventh: 0 -> (0 , 0)
	Knight Attackers: Num: 0 , Val: (0 , 0)
	Bishop Attackers: Num: 0 , Val: (0 , 0)
	Rook Attackers: Num: 0 , Val: (0 , 0)
	Queen Attackers: Num: 0 , Val: (0 , 0)
	Sum Attackers: (Num: 0 , Val: (0 , 0)
	Attack MG value: 0 * 0 / 100.0 -> 0
	Attack EG value: 0 * -1 / 100.0 -> 0
Sum: (-130 , -26)

Piecewise Sum: (-130 , -26) - (-130 , -26) -> (0 , 0)

King for White:
	Shield pawn missing: 0 -> (5 , -16)
	Shield pawn on open file missing: 0 -> (2 , 7)
Sum: (7 , -9)

King for Black:
	Shield pawn missing: 0 -> (5 , -16)
	Shield pawn on open file missing: 0 -> (2 , 7)
Sum: (7 , -9)

King Sum: (7 , -9) - (7 , -9) -> (0 , 0)

Sum: (0 , 0) + (0 , 0) + (0 , 0) + (0 , 0) + (0 , 0) + (0 , 0) + (10 , 15) -> (10 , 10) (EG/=1.5)
Phase: 128

Final Result: (10 * 128 + 10 * (128.0 - 128))/128.0 -> 10
```
### Perft
You can run perft on an arbitrary position. Note that if there is no king on the board for either side or the position is otherwise illegal, FabChess will crash (intended).
```
> position startpos
> perft 6
<
a2a3: 4463267
b2b3: 5310358
c2c3: 5417640
d2d3: 8073082
e2e3: 9726018
f2f3: 4404141
g2g3: 5346260
h2h3: 4463070
a2a4: 5363555
b2b4: 5293555
c2c4: 5866666
d2d4: 8879566
e2e4: 9771632
f2f4: 4890429
g2g4: 5239875
h2h4: 5385554
g1f3: 5723523
g1h3: 4877234
b1a3: 4856835
b1c3: 5708064
119060324
Time 1.222 (97430707.03764321 nps)
```
### Debug print
Use `d` for a debug print of the board
```
> position startpos
> d
<
+---+---+---+---+---+---+---+---+
| r | n | b | q | k | b | n | r |
+---+---+---+---+---+---+---+---+
| p | p | p | p | p | p | p | p |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |
+---+---+---+---+---+---+---+---+
| P | P | P | P | P | P | P | P |
+---+---+---+---+---+---+---+---+
| R | N | B | Q | K | B | N | R |
+---+---+---+---+---+---+---+---+
Castle Rights:
White Kingside: true
White Queenside: true
Black Kingside: true
Black Queenside: true
En Passant Possible: 0
Half-Counter: 0
Full-Counter: 1
Side to Move: 0
Hash: 6214150092099736431

FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
```
## Inspired heavily by:

- https://www.chessprogramming.org/Main_Page
- Stockfish
- Ethereal
- Asymptote

Special thanks to Andrew Grant for writing [OpenBench](https://github.com/AndyGrant/OpenBench), which is used for selfplay testing, in order to incrementally improve on the last versions.
