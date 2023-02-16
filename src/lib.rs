use std::io::{stdout, Write};
use std::iter::{repeat};
use crossterm::style::{ContentStyle, Color, Attribute};

pub enum ValueDisplay
{
    None,
    CurrentValueOnly,
    CurrentAndMaxValue,
    Percentage
}

pub struct Symbol 
{
    pub symbol: char,
    pub style: Option<ContentStyle>
}

impl Symbol
{
    pub fn new(s : char) -> Symbol
    {
        Symbol {
            symbol: s,
            style: None
        }
    }
    
    pub fn with_fg(mut self, color: Color) -> Symbol
    {
        self.style = match self.style {
            Some(mut style) => {
                style.foreground_color = Some(color);
                Some(style)
            }
            None => {
                let mut cs = ContentStyle::new();
                cs.foreground_color = Some(color);
                Some(cs)
            }
        };
        self
    }
    
    pub fn with_bg(mut self, color: Color) -> Symbol
    {
        self.style = match self.style {
            Some(mut style) => {
                style.background_color = Some(color);
                Some(style)
            }
            None => {
                let mut cs = ContentStyle::new();
                cs.background_color = Some(color);
                Some(cs)
            }
        };
        self
    }

    pub fn bold(mut self) -> Symbol
    {
        self.style = match self.style {
            Some(mut style) => {
                style.attributes.set(Attribute::Bold);
                Some(style)
            }
            None => {
                let mut cs = ContentStyle::new();
                cs.attributes.set(Attribute::Bold);
                Some(cs)
            }
        };
        self
    }
    
    pub fn dim(mut self) -> Symbol
    {
        self.style = match self.style {
            Some(mut style) => {
                style.attributes.set(Attribute::Dim);
                Some(style)
            }
            None => {
                let mut cs = ContentStyle::new();
                cs.attributes.set(Attribute::Dim);
                Some(cs)
            }
        };
        self
    }
}

pub struct Style
{
    pub value_display: ValueDisplay,
    pub value_suffix: Option<&'static str>,
    pub value_divisor: usize,
    pub left_cap: Symbol,
    pub right_cap: Symbol,
    pub empty_char: Symbol,
    pub done_char: Symbol,
    pub fill_chars: Vec<Symbol>
}

impl Style
{   
    pub fn default_ascii() -> Style {
        Style {
            value_display: ValueDisplay::None,
            value_suffix: None,
            value_divisor: 1,
            left_cap: Symbol::new('[').dim(),
            right_cap: Symbol::new(']').dim(),
            empty_char: Symbol::new(' '),
            done_char: Symbol::new('=').with_fg(Color::Green).bold(),
            fill_chars: vec![
                Symbol::new('.').with_fg(Color::Green), 
                Symbol::new(',').with_fg(Color::Green), 
                Symbol::new('-').with_fg(Color::Green), 
                Symbol::new('=').with_fg(Color::Green)]
        }
    }

    pub fn new_smooth_unicode() -> Style {
        Style {
            value_display: ValueDisplay::None,
            value_suffix: None,
            value_divisor: 1,
            left_cap: Symbol::new('\u{2595}'),   // Right 1/8th block
            right_cap: Symbol::new('\u{258f}'),  // Left 1/8th block
            empty_char: Symbol::new('\u{2588}').with_fg(Color::Blue),
            // Full block
            done_char: Symbol::new('\u{2588}').with_fg(Color::Yellow),  
            fill_chars: vec![
                Symbol::new('\u{258f}').with_fg(Color::Yellow).with_bg(Color::Blue), 
                Symbol::new('\u{258d}').with_fg(Color::Yellow).with_bg(Color::Blue), 
                Symbol::new('\u{258b}').with_fg(Color::Yellow).with_bg(Color::Blue), 
                Symbol::new('\u{2589}').with_fg(Color::Yellow).with_bg(Color::Blue)]
        }
    }

    pub fn new_climbing_blocks_unicode() -> Style {
        Style {
            value_display: ValueDisplay::None,
            value_suffix: None,
            value_divisor: 1,
            left_cap: Symbol::new('\u{2595}'),   // Right 1/8th block
            right_cap: Symbol::new('\u{258f}'),  // Left 1/8th block
            empty_char: Symbol::new('\u{2588}').with_fg(Color::DarkGrey),
            done_char: Symbol::new('\u{2588}').with_fg(Color::Magenta),  // Full block
            fill_chars: vec![
                Symbol::new('\u{2598}').with_fg(Color::Magenta).with_bg(Color::DarkGrey), 
                Symbol::new('\u{259a}').with_fg(Color::Magenta).with_bg(Color::DarkGrey), 
                Symbol::new('\u{2599}').with_fg(Color::Magenta).with_bg(Color::DarkGrey), 
                Symbol::new('\u{2588}').with_fg(Color::Magenta).with_bg(Color::DarkGrey)]
        }
    }
}

pub struct Progresso
{
    style: Style,
    curr_val : u64,
    max_val : u64,
    line_length: usize,
}

pub trait ProgressoBar
{
    fn get_total(&self) -> u64;
    fn set_total(&mut self, max : u64);

    fn get_value(&self) -> u64;
    fn set_value(&mut self, val : u64);
    
    fn get_display_len(&self) -> usize;
    fn set_display_len(&mut self, val : usize);

    // fn get_percent_done() -> f32;

    // fn tick();
    // fn tick_by_amount(amount : u64);

    fn erase(&self);
    fn erase_to(&self, writer: &mut dyn Write);

    // fn redraw();
    fn draw(&self);
    fn draw_to(&self, writer: &mut dyn Write);
}

impl Progresso
{
    pub fn new(style: Style) -> Progresso
    {
        let p = Progresso {
            style: style, 
            curr_val: 0,
            max_val: 10,
            line_length: 40, 
        };
        return p
    }

    fn percent_done(&self) -> f32 {
        return self.curr_val as f32 / self.max_val as f32
    }

    fn char_amounts(&self) -> (usize, bool, usize, f32) {
        let pct_done = self.percent_done();
        let prog_width = pct_done * self.line_length as f32;

        // truncate to get full number of done chars.
        let num_done = (prog_width) as usize;

        // Take fractional amount and scale by 
        let has_partial = if self.style.fill_chars.len() > 0 { 
            // let min_frac = 1.0 / self.style.fill_chars.len() as f32 - f32::EPSILON;
            prog_width.fract() >= f32::EPSILON
        } else {
            false
        };

        let partial_index = (prog_width.fract() * self.style.fill_chars.len() as f32) as usize;

        (num_done, has_partial, partial_index, pct_done)
    }
}

impl ProgressoBar for Progresso
{
    fn get_total(&self) -> u64 { return self.max_val; }
    fn set_total(&mut self, max : u64) { self.max_val = max; }

    fn get_value(&self) -> u64 { return self.curr_val; }
    fn set_value(&mut self, val : u64) { self.curr_val = val; }
    
    fn get_display_len(&self) -> usize { return self.line_length; }
    fn set_display_len(&mut self, val : usize) { self.line_length = val; }

    fn erase(&self) {
        let mut stdout_buf = stdout().lock();
        self.erase_to(&mut stdout_buf);
    }

    fn erase_to(&self, writer: &mut dyn Write) {
        writer.write("\r".as_bytes()).unwrap();
    }

    fn draw(&self)
    {
        let mut stdout_buf = stdout().lock();
        self.draw_to(&mut stdout_buf);
    }

    fn draw_to(&self, writer: &mut dyn Write)
    {
        match self.style.left_cap.style {
            Some(style) => write!(writer, "{}", style.apply(self.style.left_cap.symbol)).unwrap(),
            None => {
                write!(writer, "{}", self.style.left_cap.symbol).unwrap()
            }
        };
        
        
        let (num_done, has_partial, partial_index, pct_done) = self.char_amounts();
        
        let done_str = repeat(self.style.done_char.symbol).take(num_done).collect::<String>();
        match self.style.done_char.style {
            Some(style) => {
                write!(writer, "{}", style.apply(done_str));
            },
            None => {
                write!(writer, "{}", done_str);
            }
        };

        // Write the partially complete character.
        if has_partial {
            let curr_partial = &self.style.fill_chars[partial_index];
            match curr_partial.style {
                Some(style) => {
                    write!(writer, "{}", style.apply(curr_partial.symbol));
                },
                None => {
                    write!(writer, "{}", curr_partial.symbol);
                }
            };
        }

        let mut num_empty = (self.line_length - num_done) as usize;
        if has_partial {
            num_empty -= 1;
        }

        let empty_str = repeat(self.style.empty_char.symbol).take(num_empty).collect::<String>();
        match self.style.empty_char.style {
            Some(style) => {
                write!(writer, "{}", style.apply(empty_str));
            },
            None => {
                write!(writer, "{}", empty_str);
            }
        };
        
        
        match self.style.right_cap.style {
            Some(style) => write!(writer, "{}", style.apply(self.style.right_cap.symbol)).unwrap(),
            None => {
                write!(writer, "{}", self.style.right_cap.symbol).unwrap()
            }
        };

        match self.style.value_display {
            ValueDisplay::Percentage => writer.write(format!(" {} %", (pct_done*100.0) as u32).as_bytes()).unwrap(),
            ValueDisplay::None => 0,
            _ => 0,
        };
        writer.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pb_output(
            pb: &mut Progresso,
            val: u64, 
            expected: &str) {

        let mut output = Vec::new();
        pb.curr_val = val;

        pb.draw_to(&mut output);
        let output_str = std::str::from_utf8(&output).expect("Not UTF-8");
        assert_eq!(expected, output_str);
    }

    
    mod no_fill_chars
    {
        use crate::*;
        use super::*;
        
        fn no_fill_style() -> Style {
            Style {
                value_display: ValueDisplay::None,
                value_suffix: None,
                value_divisor: 1,
                left_cap: Symbol::new('['),
                right_cap: Symbol::new(']'),
                empty_char: Symbol::new('.'),
                done_char: Symbol::new('='),
                fill_chars: vec![]
            }
        }

        #[test]
        fn check_empty_ascii_pb() {
            let mut pb = Progresso::new(no_fill_style());
            test_pb_output(&mut pb, 
                0, "[........................................]"
            );
        }
        
        #[test]
        fn check_half_full_ascii_pb() {
            let mut pb = Progresso::new(no_fill_style());
            test_pb_output(&mut pb, 
                5, "[====================....................]"
            );
        }
        
        #[test]
        fn check_full_ascii_pb() {
            let mut pb = Progresso::new(no_fill_style());
            test_pb_output(&mut pb, 
                10, "[========================================]"
            );
        }
    }   
    
    mod with_fill_chars
    {
        use crate::*;
        use super::*;

        fn no_fill_style() -> Style {
            Style {
                value_display: ValueDisplay::None,
                value_suffix: None,
                value_divisor: 1,
                left_cap: Symbol::new('['),
                right_cap: Symbol::new(']'),
                empty_char: Symbol::new('.'),
                done_char: Symbol::new('='),
                fill_chars: vec![Symbol::new('-')]
            }
        }
        
        #[test]
        fn check_empty_ascii_pb() {
            let mut pb = Progresso::new(no_fill_style());
            pb.line_length = 4;
            pb.max_val = 20;

            test_pb_output(&mut pb, 
                0, "[....]"
            );
        }
        
        #[test]
        fn check_half_full_ascii_pb() {
            let mut pb = Progresso::new(no_fill_style());
            pb.line_length = 4;
            pb.max_val = 20;

            test_pb_output(&mut pb, 
                6, "[=-..]"
            );
        }
        
        #[test]
        fn check_full_ascii_pb() {
            let mut pb = Progresso::new(no_fill_style());
            pb.line_length = 4;
            pb.max_val = 20;

            test_pb_output(&mut pb, 
                20, "[====]"
            );
        }
    }
}
