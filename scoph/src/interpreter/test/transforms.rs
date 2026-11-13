use super::*;
use anyhow::Result;
use pretty_assertions::assert_eq;

// transform tests
#[test]
fn test_rotate_slant_match() -> Result<()> {
    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_file(
        "../games/tests/win-transforms.json",
    )?);

    intrp.run_full(&mut Dummy)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));

    Ok(())
}

#[test]
fn test_two_child_rotate_slant_match() -> Result<()> {
    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_file(
        "../games/tests/two-child-transforms.json",
    )?);

    intrp.run_full(&mut Dummy)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));

    Ok(())
}

#[test]
fn test_x_ident() -> Result<()> {
    const UNUSED_GROUP: &str = r#"
    {"name":"poop","desc":"","tree":{"type":"order","children":[{"type":"set-board","comment":"","nid":"","pattern":{"main":[["T","R","R","B","T"]]}},{"type":"win","comment":"","nid":"","children":[{"type":"x-ident","comment":"","nid":"unused","children":[{"type":"match","pattern":{"main":[["NOT","HERE"]]}},{"type":"match","pattern":{"main":[["NOT","HERE"]]}},{"type":"match","pattern":{"main":[["NOT","HERE"]]}},{"type":"match","pattern":{"main":[["NOT","HERE"]]}},{"type":"match","pattern":{"main":[["NOT","HERE"]]}},{"type":"match","pattern":{"main":[["T","R","R","B","T"]]}}]}],"pid":"Jeremy"}],"comment":"","nid":""}}
    "#;

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(UNUSED_GROUP)?);
    intrp.run_full(&mut Dummy)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));
    Ok(())
}

#[test]
fn test_x_spin() -> Result<()> {
    const X_SPIN_WIN: &str = r#"
     {"name":"rotate-all-test","desc":"","tree":{"type":"order","children":[{"type":"set-board","comment":"","nid":"","pattern":{"main":[["C","F","I"],["B","E","H"],["A","D","G"]]}},{"type":"win","comment":"","nid":"","children":[{"type":"x-spin","comment":"","nid":"","children":[{"type":"match","pattern":{"main":[["A","B","C"],["D","E","F"],["G","H","I"]]}}],"remorig":false}],"pid":"Jeremy"}],"comment":"","nid":""}}
    "#;

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(X_SPIN_WIN)?);
    intrp.run_full(&mut Dummy)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));
    Ok(())
}

#[test]
fn test_win_with_only_transforms() -> Result<()> {
    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_file(
        "../games/tests/win-with-only-transforms.json",
    )?);
    intrp.run::<_, StackTraceOnly>(&mut Printer)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));
    Ok(())
}

#[test]
fn test_x_link_with_transform() -> Result<()> {
    const X_LINK_WIN: &str = r#"
     {
         "name":"rotate-all-test (from template)",
         "desc":"",
         "tree":{
             "type":"order",
             "children":[
                 {"type":"set-board","comment":"","nid":"","pattern":{"main":[["C",".","."],["B","F","."],["A","E","I"],[".","D","H"],[".",".","G"]]}},
                 {
                     "type":"win",
                     "comment":"",
                     "nid":"",
                     "children":[
                         {
                             "type":"x-ident",
                             "comment":"",
                             "nid":"rotations",
                             "children":[
                                 {
                                     "type":"x-spin",
                                     "comment":"",
                                     "nid":"",
                                     "children":[{"type":"match","pattern":{"main":[["A","B","C"],["D","E","F"],["G","H","I"]]}}],
                                     "remorig":false
                                 }
                             ]
                         },
                         {
                             "type":"x-skew",
                             "comment":"",
                             "nid":"",
                             "children":[
                                 {"type":"x-link","comment":"","nid":"","target":"rotations"}
                             ],
                             "remorig":false
                         }
                     ],
                     "pid":"Jeremy"
                 }
             ],
             "comment":"",
             "nid":""
         }
     }
    "#;

    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_text(X_LINK_WIN)?);
    intrp.run_full(&mut Dummy)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));

    Ok(())
}

#[test]
fn test_x_swap_only_with_x_link() -> Result<()> {
    let mut intrp = Interpreter::new(BehaviorTree::load_from_json_file(
        "../games/tests/x-swap-with-x-link.json",
    )?);
    intrp.run::<_, StackTraceOnly>(&mut Printer)?;

    assert_eq!(intrp.outcome(), Some(&Outcome::Win("Jeremy".into())));
    Ok(())
}

use crate::prelude::*;
