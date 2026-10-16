use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::Text,
    widgets::{StatefulWidget, Widget},
};
use scoph::{
    common::{NodeAction, TRRBTPattern},
    interpreter::BehaviorTree,
    prelude::NodeIndex,
};
use tui_tree_widget::{Block, Tree, TreeItem, TreeState};

pub struct TuiTrrbtTree<'a>(TreeItem<'a, String>, Vec<String>);

impl<'a> Widget for TuiTrrbtTree<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let a = [self.0];
        let t = Tree::new(&a).unwrap();
        Widget::render(t, area, buf);
    }
}

impl StatefulWidget for TuiTrrbtTree<'_> {
    type State = TreeState<String>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let a = [self.0];
        let t = Tree::new(&a)
            .unwrap()
            .block(Block::bordered().title("Game tree"))
            .highlight_symbol("(🐠>");
        state.open(self.1);
        StatefulWidget::render(t, area, buf, state);
    }
}

impl<'a> TryFrom<&'a BehaviorTree> for TuiTrrbtTree<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a BehaviorTree) -> Result<Self, Self::Error> {
        let root = value.root();
        let root_id = value.get_node_info(root).id();
        let (leaf_text, color) = node_action_to_text(value.get_node_action(root), root_id);
        let mut ti = TreeItem::new_leaf(root_id.to_string(), Text::styled(leaf_text, color));

        let mut v = vec![root_id.to_string()];
        fill_out_ttt(&mut ti, &mut v, value, root)?;

        Ok(Self(ti, v))
    }
}

fn node_action_to_text(action: &NodeAction, id: &str) -> (String, Color) {
    let action_name = action.to_string();
    let (params, color) = match action {
        NodeAction::XMirror { remorig } => (format!("remorig: {remorig}"), Color::Gray),
        NodeAction::XSkew { remorig } => (format!("remorig: {remorig}"), Color::Gray),
        NodeAction::XRotate { remorig } => (format!("remorig: {remorig}"), Color::Gray),
        NodeAction::XSpin { remorig } => (format!("remorig: {remorig}"), Color::Gray),
        NodeAction::XFlip { remorig } => (format!("remorig: {remorig}"), Color::Gray),
        NodeAction::XSwap { what, with } => {
            (format!("what: {what:?}, with: {with:?}"), Color::Gray)
        }
        NodeAction::XReplace { what, withs } => {
            (format!("what: {what:?}, withs: {withs:?} "), Color::Gray)
        }
        NodeAction::XUnrollReplace { what } => (format!("what: {what:?}"), Color::Gray),
        NodeAction::XLink { target } => (format!("target: {target:?}"), Color::Gray),
        NodeAction::Player { pid } => (format!("pid: {pid:?}"), Color::Blue),
        NodeAction::Win { pid } => (format!("pid: {pid:?}"), Color::LightRed),
        NodeAction::Lose { pid } => (format!("pid: {pid:?}"), Color::LightRed),
        NodeAction::LoopTimes { times } => (format!("times: {}", *times), Color::LightYellow),
        NodeAction::SetBoard { pattern } => {
            (pattern_params(&["pattern"], &[pattern]), Color::LightBlue)
        }
        NodeAction::AppendRows { pattern } => {
            (pattern_params(&["pattern"], &[pattern]), Color::LightBlue)
        }
        NodeAction::AppendColumns { pattern } => {
            (pattern_params(&["pattern"], &[pattern]), Color::LightBlue)
        }
        NodeAction::Rewrite { lhs, rhs } => (
            pattern_params(&["lhs", "rhs"], &[lhs, rhs]),
            Color::LightBlue,
        ),
        NodeAction::RewriteAll { lhs, rhs } => (
            pattern_params(&["lhs", "rhs"], &[lhs, rhs]),
            Color::LightBlue,
        ),
        NodeAction::LayerTemplate { layer, with } => (
            format!("layer: {layer:?}, with: {with:?}"),
            Color::LightBlue,
        ),
        NodeAction::Match { pattern } => (pattern_params(&["pattern"], &[pattern]), Color::Cyan),
        NodeAction::MatchTimes { times, pattern } => {
            let mut out = format!("times: {}\n", *times);
            out.push_str(&pattern_params(&["pattern"], &[pattern]));
            (out, Color::Cyan)
        }
        NodeAction::Order => (String::new(), Color::Yellow),
        NodeAction::XIdent => (String::new(), Color::Gray),
        NodeAction::XPrune => (String::new(), Color::Gray),
        NodeAction::Draw => (String::new(), Color::Blue),
        NodeAction::All => (String::new(), Color::Yellow),
        NodeAction::None => (String::new(), Color::Yellow),
        NodeAction::RandomTry => (String::new(), Color::Cyan),
        NodeAction::LoopUntilAll => (String::new(), Color::Yellow),
    };

    let out = format!("({id}) {action_name}\n{params}");
    (out, color)
}

fn pattern_params(names: &[&str], patterns: &[&TRRBTPattern]) -> String {
    assert!(names.len() == patterns.len());
    let mut out = String::new();
    for (n, p) in names.into_iter().zip(patterns.into_iter()) {
        out.push_str(format!("{n}:\n").as_str());
        out.push_str(&p.to_string());
        out.push_str(" \n");
    }
    out
}

fn fill_out_ttt(
    ti: &mut TreeItem<String>,
    v: &mut Vec<String>,
    bt: &BehaviorTree,
    current: NodeIndex,
) -> anyhow::Result<()> {
    for (index, child) in bt.get_node_neighbors(current).enumerate() {
        let id = bt.get_node_info(child).id();
        v.push(id.to_string());
        let (text, color) = node_action_to_text(bt.get_node_action(child), id);
        ti.add_child(TreeItem::new_leaf(
            id.to_string(),
            Text::styled(text, color),
        ))?;
        fill_out_ttt(ti.child_mut(index).unwrap(), v, bt, child)?;
    }

    Ok(())
}
