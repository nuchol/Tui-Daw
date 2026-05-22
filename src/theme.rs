use std::{collections::HashMap, str::FromStr};
use serde::Deserialize;
use strum::{Display, EnumCount, EnumIter, EnumString, IntoEnumIterator};

use ratatui::{
    layout::{Direction, Rect}, style::{Color, Modifier, Style}, text::Line, widgets::{Block, BorderType, Borders}
};

#[derive(Copy, Clone, Hash, Eq, PartialEq,
    EnumCount, EnumIter, EnumString, Display)]
pub enum ThemeKey {
    // Global
    Normal, NormalFloat, StatusLine, StatusLineNC, ErrorMsg, WarnMsg, Cursor,
    // Float
    FloatBoarder, FloatBoarderNC, FloatTitle, FloatTitleNC, FloatFooter,
    // Modes
    ModeNormal, ModeInsert, ModeCommand,
    // File Tree
    FileTreeDir, FileTreeWindow,
    // Piano roll
    PianoRollNote, PianoRollNoteAccent, PianoRollNoteSelected,
    PianoRollBlackKey, PianoRollWhiteKey,
    PianoRollBlackKeyPressed, PianoRollWhiteKeyPressed,
    PainoRollBarSeparator, PainoRollBeatSeparator, PainoRollSubDivSeparator,
}

#[derive(Clone, Default)]
pub struct StyleDef {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifiers: Modifier,
    pub link: Option<ThemeKey>,
}

impl StyleDef {
    pub fn to_style(&self) -> Style {
        let mut s = Style::default().add_modifier(self.modifiers);
        if let Some(fg) = self.fg { s = s.fg(fg); }
        if let Some(bg) = self.bg { s = s.bg(bg); }
        s
    }
}

#[derive(Deserialize)]
struct ThemeFile {
    palette: HashMap<String, String>,
    options: HashMap<String, String>,
    group: HashMap<String, RawGroup>,
}

#[derive(Deserialize, Default)]
struct RawGroup {
    fg:         Option<String>,
    bg:         Option<String>,
    bold:       Option<bool>,
    italic:     Option<bool>,
    dim:        Option<bool>,
    reversed:   Option<bool>,
    link:       Option<String>,

    bright:     Option<bool>,
}

pub struct ThemeRegistry {
    base: HashMap<ThemeKey, StyleDef>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self { base: HashMap::new() }
    }

    pub fn from_toml(src: &str) -> Result<Self, String> {
        let mut base = HashMap::new();

        let file: ThemeFile = toml::from_str(src).map_err(|e| e.to_string())?;

        for (name, raw) in &file.group {
            let key = ThemeKey::from_str(name).map_err(|_| format!("unknown group: {name}"));

            let fg = raw.fg.as_deref().map(|v| Self::parse_color(v, &file.palette)).transpose()?;
            let bg = raw.bg.as_deref().map(|v| Self::parse_color(v, &file.palette)).transpose()?;

            let mut modifiers = Modifier::empty();
            if raw.bold     == Some(true) { modifiers |= Modifier::BOLD; }
            if raw.italic   == Some(true) { modifiers |= Modifier::ITALIC; }
            if raw.dim      == Some(true) { modifiers |= Modifier::DIM; }
            if raw.reversed == Some(true) { modifiers |= Modifier::REVERSED; }

            let link = raw.link.as_deref()
                .map(|n| ThemeKey::from_str(n).map_err(|_| format!("unknown link target: {n}")))
                .transpose()?;

            base.insert(key?, StyleDef { fg, bg, modifiers, link });
        }

        Ok(Self { base })
    }

    fn parse_color(value: &str, palette: &HashMap<String, String>) -> Result<Color, String> {
        let hex = if let Some(key) = value.strip_prefix("p:") {
            palette.get(key).ok_or_else(|| format!("unknown palette key: {key}"))?
        } else {
            value
        };

        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!("invalid hex color: #{hex}"));
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
        Ok(Color::Rgb(r, g, b))
    }

    pub fn set(&mut self, key: ThemeKey, def: StyleDef) {
        self.base.insert(key, def);
    }

    pub fn resolve(&self, key: ThemeKey) -> Style {
        self.resolve_inner(key, 0)
    }

    fn resolve_inner(&self, key: ThemeKey, depth: u8) -> Style {
        // cycle guard
        if depth > 8 { return Style::default(); }

        match self.base.get(&key) {
            None => Style::default(),
            Some(d) if d.link.is_some() => self.resolve_inner(d.link.unwrap(), depth + 1),
            Some(d) => d.to_style(),
        }
    }
}

pub struct ResolvedTheme {
    styles: [Style; ThemeKey::COUNT as usize]
}

impl ResolvedTheme {
    pub const BORDER_TYPE: BorderType = BorderType::Rounded;

    pub fn from_registry(reg: &ThemeRegistry) -> Self {
        let mut styles = [Style::default(); ThemeKey::COUNT];

        for key in ThemeKey::iter() {
            styles[key as usize] = reg.resolve(key);
        }

        Self { styles }
    }

    #[inline]
    pub fn get(&self, key: ThemeKey) -> Style {
        self.styles[key as usize]
    }

    pub fn window_border<'a>(&self, title: &'a str, focused: bool) -> Block<'a> {
        let style = if focused {
            self.get(ThemeKey::FloatBoarder)
        } else {
            self.get(ThemeKey::FloatBoarderNC)
        };

        Block::bordered()
            .title(Line::styled(title, style).centered())
            .borders(Borders::ALL)
            .border_type(Self::BORDER_TYPE)
            .border_style(style)
    }

    pub fn divider(&self, dir: &Direction) -> Block<'_> {
        Block::bordered().borders(if *dir == Direction::Vertical
            { Borders::TOP } else { Borders::LEFT })
            .border_style(self.get(ThemeKey::FloatBoarder))
    }

    pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let width = (area.width * percent_x) / 100;
        let height = (area.height * percent_y) / 100;
        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;

        Rect { x, y, width, height }
    }
}
