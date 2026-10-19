#include "raylib.h"
#include "scoph.h"
#include <stdio.h>
#include <stdlib.h>

#define DEFAULT_LAYER "main"

const int cell_size = 60;
const int sw = 800;
const int sh = 450;

void fail_and_print(ScophError code, ScophIntrp *intrp,
                    const char *additional_info) {
  printf("Scoph Error: %s. Context: %s\n", scoph_strerror(code),
         additional_info);
  scoph_intrp_free(&intrp);
  exit(1);
}

void draw_board(ScophIntrp *intrp, ScophIntrpResult last_res) {
  unsigned int width, height;
  ScophError code =
      scoph_board_layer_size(intrp, DEFAULT_LAYER, &width, &height);
  switch (code) {
  case Ok:
    break;
  case MissingLayerOnSrc:
    DrawText("No board yet", sw / 2, sh / 2, 14, BLACK);
    return; // it can be assumed that we might not have the layer to start
  default:
    fail_and_print(code, intrp,
                   "Error happened while trying to get the board size");
  }

  // draw base board
  unsigned int board_width = width * cell_size;
  unsigned int board_height = height * cell_size;
  const int board_x = cell_size + 50;
  const int board_y = cell_size + 15;
  DrawRectangle(board_x, board_y, board_width, board_height, GRAY);

  // draw lines and cell characters
  for (int y = 0; y < height; y++) {
    int liney = board_y + y * (cell_size + 1);
    DrawLine(board_x, liney, board_x + board_width, liney, BLACK);
    for (int x = 0; x < width; x++) {
      int linex = board_x + x * (cell_size + 1);
      DrawLine(linex, board_y, linex, board_y + board_height, BLACK);

      char cellbuf[8];
      unsigned int bytes_read;
      ScophError code = scoph_view_board_cell(intrp, DEFAULT_LAYER, x, y,
                                              cellbuf, &bytes_read);
      cellbuf[bytes_read] = '\0';
      if (code) {
        fail_and_print(
            code, intrp,
            "Error happened while trying to read the board contents");
      }

      int text_x = board_x + cell_size / 10 +  x * cell_size;
      int text_y = board_y + y * cell_size;
      DrawText(cellbuf, text_x, text_y, cell_size, BLACK);
    }
  }

  if (last_res == Finished) {
    DrawText("Interpreter is finished!", 0, 0, 15, RED);
  }
}

int main() {
  // scoph setup code
  ScophIntrp *intrp;
  ScophError code = scoph_intrp_load(&intrp, "rewrite-dots.json");
  if (code) {
    fail_and_print(code, intrp, "Error occurred while trying to load the JSON");
  }

  // raylib setup code
  InitWindow(sw, sh, "Scoph");

  SetTargetFPS(60);

  const int ticks_before_step = 20;
  int ticks = ticks_before_step;

  while (!WindowShouldClose()) {
    ScophIntrpResult res;
    if (ticks == 0) {
      ScophError code = scoph_intrp_step(intrp, &res);
      if (code) {
        fail_and_print(code, intrp,
                       "Error occured while trying to take a step");
      }
      ticks = ticks_before_step;
    } else {
      ticks--;
    }

    BeginDrawing();
    ClearBackground(RAYWHITE);
    draw_board(intrp, res);
    EndDrawing();
  }

  return 0;
}
