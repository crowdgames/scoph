#include "raylib.h"
#include "scoph.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

ScophIntrp *ttt;
ScophPlayerChoices player_choices;

#define CELL_SIZE (150.0f)
Rectangle board_rect = {150.0, 50.0, CELL_SIZE * 3, CELL_SIZE * 3.0};

typedef enum { NONE, X, O } PlayerTurn;

PlayerTurn turn = NONE;

#define TRY(s)                                                                 \
  do {                                                                         \
    ScophError code = s;                                                       \
    if (code) {                                                                \
      printf("Scoph Error (%s:%d): %s\n", __FILE__, __LINE__,                  \
             scoph_strerror(code));                                            \
      scoph_intrp_free(&ttt);                                                  \
      exit(1);                                                                 \
    }                                                                          \
  } while (0);

void on_player_choice() {
  bool is_x = strcmp(player_choices.player_id, "X") == 0;
  bool is_o = strcmp(player_choices.player_id, "O") == 0;
  if (is_x) {
    turn = X;
  } else if (is_o) {
    turn = O;
  }
}

void do_player_choice(uint32_t x, uint32_t y) {
  printf("Trying to click\n");
  for (int rewrite_index = 0;
       rewrite_index < scoph_choices_count(player_choices); ++rewrite_index) {
    for (int match_index = 0;
         match_index <
         scoph_choices_match_count(player_choices.choices, rewrite_index);
         ++match_index) {

      uint32_t mx, my;
      scoph_choices_coords(player_choices.choices, rewrite_index, match_index,
                           &mx, &my);

      if (x == mx && y == my) {
        TRY(scoph_intrp_player_choose(ttt, rewrite_index, match_index));
        turn = NONE;
        return;
      }
    }
  }
}

void draw_move(const char *content, unsigned int x, unsigned int y) {
  Color text_color;
  if (strcmp(content, "_") == 0) {
    text_color = DARKGRAY;
  } else if (strcmp(content, "X") == 0) {
    text_color = BLUE;
  } else if (strcmp(content, "O") == 0) {
    text_color = RED;
  }

  DrawTextEx(GetFontDefault(), content,
             (Vector2){board_rect.x + CELL_SIZE / 4.0 + (float)x * CELL_SIZE,
                       board_rect.y + (float)y * CELL_SIZE},
             CELL_SIZE, .0, text_color);
}

int main() {
  InitWindow(800, 600, "Tic Tac Toe");

  TRY(scoph_intrp_load(&ttt, "games/trees/tic-tac-toe.json"))

  const char *junk; // we know what the game is so we don't care
  TRY(scoph_intrp_startup(ttt, &junk, &junk))

  while (!WindowShouldClose()) {
    ScophIntrpResult res;
    TRY(scoph_intrp_step(ttt, &res))

    if (res == WaitingForChoice && turn == NONE) {
      TRY(scoph_intrp_check_choices(ttt, &player_choices));
      on_player_choice();
    }

    Vector2 mp = GetMousePosition();
    bool hover_board = CheckCollisionPointRec(mp, board_rect);

    Vector2 grid_pos = {
        (int)((mp.x - board_rect.x) / CELL_SIZE),
        (int)((mp.y - board_rect.y) / CELL_SIZE),
    };

    if (turn != NONE && hover_board &&
        IsMouseButtonPressed(MOUSE_BUTTON_LEFT)) {
      do_player_choice((uint32_t)grid_pos.x, (uint32_t)grid_pos.y);
    }

    BeginDrawing();
    ClearBackground(WHITE);
    DrawRectangleRec(board_rect, GRAY);

    for (int i = 0; i <= 3; ++i) {
      float offset = board_rect.width * ((float)i / 3.0);
      // vertical line
      DrawLineEx((Vector2){board_rect.x + offset, board_rect.y},
                 (Vector2){board_rect.x + offset,
                           (int)board_rect.y + board_rect.height},
                 5.0, BLACK);
      // horizontal line
      DrawLineEx(
          (Vector2){board_rect.x, board_rect.y + offset},
          (Vector2){board_rect.x + board_rect.width, board_rect.y + offset},
          5.0, BLACK);
    }

    if (hover_board) {
      Color cell_color;
      switch (turn) {
      case NONE:
        cell_color = BLACK;
        break;
      case X:
        cell_color = BLUE;
        break;
      case O:
        cell_color = RED;
        break;
      }

      DrawRectangleLinesEx(board_rect, 10.0, BLACK);
      DrawRectangleLinesEx((Rectangle){board_rect.x + grid_pos.x * CELL_SIZE,
                                       board_rect.y + grid_pos.y * CELL_SIZE,
                                       CELL_SIZE, CELL_SIZE},
                           10.0, cell_color);
    }

    // draw board contents
    unsigned int board_cellw, board_cellh;
    TRY(scoph_board_size(ttt, &board_cellw, &board_cellh))

    for (int y = 0; y < board_cellh; ++y) {
      for (int x = 0; x < board_cellh; ++x) {
        const char *content;
        TRY(scoph_get_board_cell(ttt, "main", &content, x, y))
        draw_move(content, x, y);
      }
    }
    EndDrawing();
  }

  scoph_intrp_free(&ttt);
}
