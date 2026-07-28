import React, { useEffect, useRef, useState } from "react";

const COLS = 20;
const ROWS = 20;
const BLOCK = 30;

const COLORS = {
  I: "#00e5ff",
  J: "#2979ff",
  L: "#ff9100",
  O: "#ffea00",
  S: "#00e676",
  T: "#d500f9",
  Z: "#ff1744",
};

const SHAPES = {
  I: [
    [0,0,0,0],
    [1,1,1,1],
    [0,0,0,0],
    [0,0,0,0]
  ],

  J: [
    [1,0,0],
    [1,1,1],
    [0,0,0]
  ],

  L: [
    [0,0,1],
    [1,1,1],
    [0,0,0]
  ],

  O: [
    [1,1],
    [1,1]
  ],

  S: [
    [0,1,1],
    [1,1,0],
    [0,0,0]
  ],

  T: [
    [0,1,0],
    [1,1,1],
    [0,0,0]
  ],

  Z: [
    [1,1,0],
    [0,1,1],
    [0,0,0]
  ]
};

function createBoard() {
  return Array.from({ length: ROWS }, () =>
    Array(COLS).fill(null)
  );
}

function rotate(matrix) {
  return matrix[0].map((_, i) =>
    matrix.map(row => row[i]).reverse()
  );
}

export default function Tetris() {

  const canvasRef = useRef();

  const [board, setBoard] = useState(createBoard());

  const [piece, setPiece] = useState(null);

  const [score, setScore] = useState(0);

  const [gameOver, setGameOver] = useState(false);

  function randomPiece() {

    const keys = Object.keys(SHAPES);

    const type =
      keys[Math.floor(Math.random() * keys.length)];

    return {

      x: 3,

      y: 0,

      type,

      color: COLORS[type],

      shape:
        SHAPES[type].map(r => [...r])

    };
  }

  function collision(p, b = board) {

    for (let y = 0; y < p.shape.length; y++) {

      for (let x = 0; x < p.shape[y].length; x++) {

        if (!p.shape[y][x]) continue;

        const newX = p.x + x;

        const newY = p.y + y;

        if (
          newX < 0 ||
          newX >= COLS ||
          newY >= ROWS
        )
          return true;

        if (
          newY >= 0 &&
          b[newY][newX]
        )
          return true;
      }
    }

    return false;
  }

  function merge() {

    const newBoard =
      board.map(r => [...r]);

    piece.shape.forEach((row, y) => {

      row.forEach((v, x) => {

        if (v) {

          newBoard[piece.y + y][piece.x + x] =
            piece.color;

        }

      });

    });

    let cleared = 0;

    for (let y = ROWS - 1; y >= 0; y--) {

      if (
        newBoard[y].every(c => c)
      ) {

        newBoard.splice(y, 1);

        newBoard.unshift(
          Array(COLS).fill(null)
        );

        cleared++;

        y++;

      }

    }

    setScore(s => s + cleared * 100);

    setBoard(newBoard);

    const next = randomPiece();

    if (collision(next, newBoard)) {

      setGameOver(true);

    }

    setPiece(next);

  }

  function move(dx, dy) {

    if (gameOver) return;

    const next = {

      ...piece,

      x: piece.x + dx,

      y: piece.y + dy

    };

    if (!collision(next)) {

      setPiece(next);

    } else if (dy === 1) {

      merge();

    }

  }

  function hardDrop() {

    let p = { ...piece };

    while (!collision({ ...p, y: p.y + 1 })) {

      p.y++;

    }

    setPiece(p);

    setTimeout(merge);

  }

  function rotatePiece() {

    const next = {

      ...piece,

      shape: rotate(piece.shape)

    };

    if (!collision(next))

      setPiece(next);

  }

  useEffect(() => {

    setPiece(randomPiece());

  }, []);

  useEffect(() => {

    if (!piece || gameOver) return;

    const id = setInterval(() => {

      move(0,1);

    },500);

    return () => clearInterval(id);

  });

  useEffect(() => {

    function key(e){

      if(!piece) return;

      if(e.key==="ArrowLeft")
        move(-1,0);

      if(e.key==="ArrowRight")
        move(1,0);

      if(e.key==="ArrowDown")
        move(0,1);

      if(e.key==="ArrowUp")
        rotatePiece();

      if(e.code==="Space")
        hardDrop();

    }

    window.addEventListener(
      "keydown",
      key
    );

    return ()=>
      window.removeEventListener(
        "keydown",
        key
      );

  },[piece]);

  useEffect(() => {

    const ctx =
      canvasRef.current.getContext("2d");

    ctx.fillStyle="#111";

    ctx.fillRect(
      0,
      0,
      COLS*BLOCK,
      ROWS*BLOCK
    );

    board.forEach((row,y)=>{

      row.forEach((c,x)=>{

        if(c){

          ctx.fillStyle=c;

          ctx.fillRect(
            x*BLOCK,
            y*BLOCK,
            BLOCK,
            BLOCK
          );

          ctx.strokeStyle="#222";

          ctx.strokeRect(
            x*BLOCK,
            y*BLOCK,
            BLOCK,
            BLOCK
          );

        }

      });

    });

    if(piece){

      piece.shape.forEach((row,y)=>{

        row.forEach((v,x)=>{

          if(v){

            ctx.fillStyle=
              piece.color;

            ctx.fillRect(
              (piece.x+x)*BLOCK,
              (piece.y+y)*BLOCK,
              BLOCK,
              BLOCK
            );

            ctx.strokeStyle="#222";

            ctx.strokeRect(
              (piece.x+x)*BLOCK,
              (piece.y+y)*BLOCK,
              BLOCK,
              BLOCK
            );

          }

        });

      });

    }

  },[board,piece]);

  return (

    <div
      style={{
        display:"flex",
        flexDirection:"column",
        alignItems:"center",
        color:"white"
      }}
    >

      <h3>Score: {score}</h3>

      {gameOver &&
        <h2>Game Over</h2>
      }

      <canvas
        ref={canvasRef}
        width={COLS*BLOCK}
        height={ROWS*BLOCK}
        style={{
          border:"2px solid white"
        }}
      />

      <p>
        ← → Move | ↑ Rotate | ↓ Drop | Space Hard Drop
      </p>

    </div>

  );

}