#ifndef scoph_h
#define scoph_h

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Represents an error may encounter when running TRRBT code.
 */
typedef enum ScophError {
  /**
   * No error, can continue
   */
  Ok,
  /**
   * Json file was not parseable for interpreter
   */
  BadJson,
  /**
   * Failed to load a provided json file
   */
  LoadFailed,
  /**
   * A node in the tree is missing a child at a specific index
   */
  MissingChild,
  /**
   * Attempt to access a layer on a pattern that doesn't exist.
   */
  MissingLayerOnSrc,
  /**
   * Attempt to write a pattern to a layer on a pattern that doesn't exists.
   */
  MissingLayerOnDest,
  /**
   * Attempt to reference a node with an id that doesn't exist in the tree.
   */
  MissingNodeId,
  /**
   * Tried getting checking the outcome of an interpreter while it's unfinished.
   */
  NoOutcome,
  /**
   * Tried calling startup again after the interpreter was already initialized
   */
  AlreadyStarted,
  /**
   * Tried calling step before the interpreter was initialized with startup
   */
  HasntStarted,
  /**
   * Tree contains an x-link node to a target id that doesn't exist in the tree
   */
  BadLink,
  /**
   * A player node in the tree has a child that is neither transform nor rewrite
   */
  BadPlayerChild,
  /**
   * Tried to make a choice when the interpreter doesn't have any
   */
  NoChoices,
  /**
   * Tried to make a choice at a coord where a match doesn't exist
   */
  BadChoice,
} ScophError;

/**
 * The current status of a scoph interpreter
 */
typedef enum ScophIntrpResult {
  /**
   * The default state of an interpreter, means the program hasn't started running yet
   */
  Unstarted,
  /**
   * The interpreter is still running and hasn't finished
   */
  InProgress,
  /**
   * The interpreter encountered an error and can no longer run
   */
  Failed,
  /**
   * The interpreter is finished running the game. Either by completing running the tree or
   * finding a winner.
   */
  Finished,
  /**
   * The interpreter has stepped into a "player" node, and the user must help the player make a
   * choice. See [scoph_view_choices] and [scoph_make_choice]
   */
  WaitingForChoice,
} ScophIntrpResult;

typedef enum ScophOutcomeType {
  Win,
  Lose,
  Tie,
} ScophOutcomeType;

typedef struct Pattern Pattern;

typedef struct ScophIntrp ScophIntrp;

/**
 * Represents the set of choices that the interpreter is waiting for the player to pick
 */
typedef struct ScophPlayerChoices ScophPlayerChoices;

typedef struct ScophOutcome {
  enum ScophOutcomeType outcome;
  /**
   * Id of the player related to the outcome. Will be "nobody" if it's a tie
   */
  const char *player_id;
} ScophOutcome;

/**
 * Reads a TRRBT json file and constructs an interpreter. The caller is responsible for freeing the
 * interpreter from memory with [`scoph_intrp_free`]. If you want to load an interpreter from json
 * text directly, see [`scoph_intrp_load_json`].
 */
enum ScophError scoph_intrp_load(struct ScophIntrp **intrp, const char *path);

/**
 * Reads TRRBT json data and constructs an interpreter. The caller is responsible for freeing the
 * interpreter from memory with [`scoph_intrp_free`]. If you want to load an interpreter from a
 * json file on the system, see [`scoph_intrp_load`].
 */
enum ScophError scoph_intrp_load_json(struct ScophIntrp **intrp, const char *json_text);

/**
 * Frees a scoph interpreter from memory.
 */
void scoph_intrp_free(struct ScophIntrp **intrp);

/**
 * Calls the startup method on the interpreter. Meant to initialize any necessary engine logic for
 * the user. Provides the name and description of the game
 */
enum ScophError scoph_intrp_startup(struct ScophIntrp *intrp, const char **name, const char **desc);

/**
 * Makes an interpreter take a step in its execution. [res] is a [ScophIntrpResult] that signifies
 * the status of the interpreter after the step.
 */
enum ScophError scoph_intrp_step(struct ScophIntrp *intrp, enum ScophIntrpResult *res);

/**
 * If an interpreter returns its finished then users can check the outcome
 */
enum ScophError scoph_get_outcome(const struct ScophIntrp *intrp, struct ScophOutcome *outcome);

/**
 * Provides the size in cells of a layer on the game's board. Will return a
 * [`ScophError::MissingLayerOnSrc`] if the layer doesn't exist on the board.
 */
enum ScophError scoph_board_size(const struct ScophIntrp *intrp,
                                 unsigned int *width,
                                 unsigned int *height);

/**
 * Gets the cell of the grid at the specified layer and coordinates. Returns a
 * [ScophError::MissingLayerSrc](crate::error::ScophError)
 */
enum ScophError scoph_get_board_cell(const struct ScophIntrp *intrp,
                                     const char *layer_name,
                                     const char **dest,
                                     unsigned int x,
                                     unsigned int y);

/**
 * Gives a more detailed string of the error code that can be printed
 */
const char *scoph_strerror(enum ScophError error);

void scoph_pattern_size(struct Pattern pat, unsigned int *width, unsigned int *height);

enum ScophError scoph_pattern_cell(struct Pattern pat,
                                   unsigned int x,
                                   unsigned int y,
                                   const char *layer_name,
                                   const char **dst);

/**
 * Get the count of possible rewrites that the player can choose
 */
unsigned int scoph_choices_count(struct ScophPlayerChoices choices);

/**
 * Get the count of matches of the left-hand side of the rewrite at the given index.
 */
unsigned int scoph_choices_match_count(struct ScophPlayerChoices choices,
                                       unsigned int rewrite_index);

/**
 * Accesses the lhs and rhs of a rewrite in a set of player choices. Will panic if the index is
 * invalid. See [scoph_choices_count].
 */
void scoph_choices_rewrites(struct ScophPlayerChoices choices,
                            unsigned int rewrite_index,
                            struct Pattern *lhs,
                            struct Pattern *rhs);

/**
 * Get the coordinates at the specified index of the specified rewrite
 */
void scoph_choices_coords(struct ScophPlayerChoices choices,
                          unsigned int rewrite_index,
                          unsigned int match_index,
                          unsigned int *x,
                          unsigned int *y);

/**
 * Get the ID of the player associated with these choices
 */
const char *scoph_choices_player_id(struct ScophPlayerChoices choices);

enum ScophError scoph_intrp_player_choose(struct ScophIntrp *intrp,
                                          unsigned int rewrite_index,
                                          unsigned int match_index);

enum ScophError scoph_intrp_check_choices(const struct ScophIntrp *intrp,
                                          struct ScophPlayerChoices *choices);

#endif  /* scoph_h */
