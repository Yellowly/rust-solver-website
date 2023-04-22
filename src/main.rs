use std::{thread::current, 
    default, 
    collections::LinkedList,
    fs::File,
    io::{self, BufRead},
    path::Path
};

use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::{JsCast, JsValue};


fn main(){
    println!("{:?}",include_str!("5letterhiddenwords.txt").split("\r\n").collect::<Vec<&'static str>>());

    //include_str!("5letterwords.txt");
    yew::start_app::<RootComponent>();
}

enum RowDataMsg{
    Data(u8),
    Request,
    Reset,
    FirstWord
}


struct RootComponent{
    words: Vec<String>,
    requesting: bool,
    solver: WordleSolver,
    prev_id: usize,
}
impl Component for RootComponent{
    type Message = RowDataMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let solver = WordleSolver::create();
        let guessed_id: usize = solver.guess_word().0;
        Self{requesting:false,words: vec![solver.get_word(guessed_id)], solver, prev_id: guessed_id}
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool{
        match msg{
            RowDataMsg::Data(d) => {
                self.requesting=false;
                self.solver.update_poss(self.prev_id, d);
                // next guess
                let guessed_id: usize = self.solver.guess_word().0;
                if self.words.len()<6 && self.solver.possibilities.len()>0
                { self.words.push(self.solver.get_word(guessed_id)); }
                else if self.solver.possibilities.is_empty(){ self.words.push(String::from("-----")) }
                self.prev_id = guessed_id;
                true
            }
            RowDataMsg::Request => {
                self.requesting=true;
                true
            }
            RowDataMsg::Reset => {
                self.solver.reset();
                let guessed_id: usize = self.solver.guess_word().0;
                //self.words=vec![self.solver.get_word(guessed_id)];
                self.words=Vec::default();
                self.prev_id = guessed_id;
                true
            }
            RowDataMsg::FirstWord => {
                self.words=vec![self.solver.get_word(self.prev_id)];
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        //let test_vals: Vec<f64> = vec![1.0,2.0,2.0,2.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,3.0,4.0,4.0];
        let test_vals: Vec<f64> = vec![1.0,1.0,2.0,2.0,2.0,3.0,3.0,3.0,3.0,4.0,4.0,4.0,5.0,5.0,6.0];
        //background, secondary, text, accent, lines
        //#505050 or #808080 #ffffff for text
        let colors = vec!["#303030".to_string(), "#404040".to_string(), "#808080".to_string(), "#e2b831".to_string(), "#000000".to_string()];
        if self.words.is_empty() { link.callback(|_| RowDataMsg::FirstWord).emit(0) }
        html!{
            <>
                <h1>{"Wordle Solver"}</h1>
                <hr/>
                <div class="input-section">
                    <div class="board-grid">
                    {
                        self.words.iter().enumerate().map(|(i,w)|{
                            html!{
                                <BoardRow word={w.clone()} state={if i==self.words.len()-1 { if self.requesting {RowState::Requesting} else {RowState::Active}} else {RowState::Static}} send_data={link.callback(move |d| RowDataMsg::Data(d))}/>
                            }
                        }).collect::<Html>()
                    }
                    </div>
                    <div class="buttons">
                        if self.solver.possibilities.len()>0 {<button class="submit" onclick={link.callback(|_| RowDataMsg::Request)}>{"Submit"}</button>}
                        <button class="submit" onclick={link.callback(|_| RowDataMsg::Reset)}>{"Reset"}</button>
                    </div>
                </div>
            </>
        }
    }
}
enum WordMsg{
    FlipLetter(usize)
}
#[derive(PartialEq, Clone)]
enum RowState{
    Active,
    Static,
    Requesting
}

#[derive(Clone, PartialEq, Properties)]
struct WordProps{
    word: String,
    send_data: Callback<u8>,
    state: RowState
}
struct BoardRow{
    letter_states: Vec<LetterState>
}
impl Component for BoardRow{
    type Message = WordMsg;
    type Properties = WordProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self{letter_states: vec![LetterState::Gray; ctx.props().word.len()]}
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool{
        match msg{
            WordMsg::FlipLetter(id) => { 
                self.letter_states[id]=LetterState::from_i8(self.letter_states[id].clone() as i8 + 1);
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        if ctx.props().state==RowState::Requesting{
            ctx.props().send_data.emit(get_pattern_from_enum(&self.letter_states));
        }
        html!{
            <div class="board-row">
            {
                ctx.props().word.chars().enumerate().map(|(i,c)|{
                    html!{
                        <LetterButton letter={c} flippable={ctx.props().state==RowState::Active} onclick={link.callback(move |_| WordMsg::FlipLetter(i))}/>
                    }
                }).collect::<Html>()
            }
            </div>
        }
    }
}

#[derive(PartialEq, Clone)]
enum LetterState{
    Gray, // 0
    Yellow, // 1
    Green // 2
}
impl ToString for LetterState{
    fn to_string(&self) -> String {
        String::from(match &self{
            LetterState::Gray => "gray",
            LetterState::Yellow => "yellow",
            LetterState::Green => "green"
        })
    }
}
impl LetterState{
    fn from_i8(num: i8) -> LetterState{
        match num%3{
            0 => LetterState::Gray,
            1 => LetterState::Yellow,
            _ => LetterState::Green
        }
    }
}
enum Msg{
    Flip
}

#[derive(Clone, PartialEq, Properties)]
struct LetterButtonProps{
    letter: char,
    flippable: bool,
    onclick: Callback<WordMsg>
}

struct LetterButton{
    rot: u32,
    state: LetterState
}

impl Component for LetterButton{
    type Message = Msg;
    type Properties = LetterButtonProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self{state: LetterState::Gray, rot:0}
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool{
        match msg{
            Msg::Flip => { 
                if _ctx.props().flippable{
                    self.rot+=180;
                    self.state=LetterState::from_i8(self.state.clone() as i8 + 1);
                    _ctx.props().onclick.emit(WordMsg::FlipLetter(0));
                    true
                }else{ false }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let flipped: bool = self.rot%360==180;
        let front_button: String;
        let back_button: String;
        if flipped{
            back_button = format!{"back-button {}",&self.state.to_string()};
            front_button = format!{"front-button {}",LetterState::from_i8(self.state.clone() as i8-1).to_string()};
        }else{
            front_button = format!{"front-button {}",&self.state.to_string()};
            back_button = format!{"back-button {}",LetterState::from_i8(self.state.clone() as i8-1).to_string()};
        }
        
        let style: String = format!("transform:rotateX({}deg)",self.rot);
        html!{
            <div class = "letter-container">
                <div class = "letter-flipper" style={style}>
                    <button class={front_button} onclick={link.callback(|_| Msg::Flip)}>{ctx.props().letter}</button>
                    <button class={back_button} onclick={link.callback(|_| Msg::Flip)}>{ctx.props().letter}</button>
                </div>
            </div>
        }
    }
}

fn get_pattern(input: &String) -> u8 {
    let mut res: u8 = 0;
    for c in input.chars().rev() {
        res *= 3_u8;
        res += match c {
            'G' => 2_u8,
            'Y' => 1_u8,
            _ => 0_u8,
        }
    }
    res
}
fn get_pattern_from_enum(input: &Vec<LetterState>) -> u8 {
    let mut res: u8 = 0;
    for c in input.iter().rev() {
        res *= 3_u8;
        res += match c {
            LetterState::Green => 2_u8,
            LetterState::Yellow => 1_u8,
            LetterState::Gray => 0_u8,
        }
    }
    res
}



struct WordleSolver{
    possibilities: LinkedList<usize>,
    og_poss: LinkedList<usize>,
    words: Vec<[u8;5]>
}

impl WordleSolver{
    fn create() -> WordleSolver{
        let words: Vec<[u8;5]> = words_to_arr(include_str!("5letterwords.txt").split("\r\n").collect::<Vec<&'static str>>());
        let poss_words: Vec<[u8;5]> = words_to_arr(include_str!("5letterhiddenwords.txt").split("\r\n").collect::<Vec<&'static str>>());
        let mut pointer: usize = 0;
        let mut possibilities: LinkedList<usize> = LinkedList::new();
        for i in 0..words.len() {
            if pointer<poss_words.len() && arr_eq(&words[i],&poss_words[pointer]) {
                possibilities.push_back(i);
                pointer+=1;
            }
        }
        return Self{og_poss: possibilities.clone(), possibilities, words}
    }
    fn reset(&mut self){
        self.possibilities=self.og_poss.clone();
    }
    fn update_poss(&mut self, guessed_word: usize, pattern: u8){
        let mut new_poss: LinkedList<usize> = LinkedList::new();
    
        //poss.drain_filter(|w| gen_pattern(&words[guessedWord as usize], &words[*w as usize]) == pattern); this function would be so useful but sadly it's only on nightly rust release
    
        for w in &self.possibilities {
            if self.gen_pattern(guessed_word, *w) == pattern {
                new_poss.push_back(*w);
            }
        }
        self.possibilities=new_poss;
    }
    //evaluates a word using information theory formula thing
    fn evaluate(&self, guess: usize) -> f32 {
        let mut num_of_poss: [u16; 243] = [0; 243];
        //stores number of possibilities of a word given a pattern, for example [0] stores all possible words if pattern was Gray Gray Gray Gray Gray
        for i in &self.possibilities {
            let pattern: u8 = self.gen_pattern(guess, i.clone());
            num_of_poss[pattern as usize] += 1;
        }
        //actually does the information theory math
        let mut res: f32 = 0.0;
        for i in num_of_poss {
            let prob: f32 = (i as f32) / (self.possibilities.len() as f32);
            if prob > 0.00000001 { res += prob * f32::log2(1.0 / prob); }
        }
        res
    }
    //guesses a word by finding the word with best eval
    fn guess_word(&self) -> (usize, f32) {
        if self.possibilities.len()>2314{
            return (8845, 1.0) //make faster
        }
        let mut best_eval: f32 = 0.0;
        let mut best_found: usize = 0;

        //random number i made up, if more than 1 remaining then finds the word with best eval, otherwise find possible word with best eval
        if self.possibilities.len() > 2 {
            for i in 0..self.words.len() {
                let eval: f32 = self.evaluate(i);
                if eval > best_eval { best_eval = eval; best_found = i; }
            }
        } else {
            for i in &self.possibilities {
                //ctrl c ctrl v lmao too lazy to make another method
                let eval: f32 = self.evaluate(i.clone());
                if eval >= best_eval { best_eval = eval; best_found = i.clone(); }
            }
        }
        (best_found, best_eval)
    }

    //generates a pattern given a guess and answer. patterns are base-3 numbers where 0 is gray, 1 is yellow, and 2 is green
    fn gen_pattern(&self, guess_id: usize, ans_id: usize) -> u8 {
        let guess: [u8; 5] = self.words[guess_id];
        let answer: [u8; 5] = self.words[ans_id];
        let mut res: u8 = 0;
        let mut chars_in_guess: [u8; 26] = [0; 26];
        let mut chars_in_answer: [u8; 26] = [0; 26];
        for i in (0..5).rev() {
            res *= 3;
            if guess[i] == answer[i] { res += 2; }
            chars_in_guess[guess[i] as usize] += 1;
            chars_in_answer[answer[i] as usize] += 1;
        }
        let mut yellows: u8 = 0;
        for i in (0..5).rev() {
            yellows *= 3;
            if guess[i] != answer[i] {
                if chars_in_answer[guess[i] as usize] < chars_in_guess[guess[i] as usize] {
                    chars_in_guess[guess[i] as usize] -= 1;
                } else { yellows += 1; }
            }
        }
        res + yellows
    }
    fn get_word(&self, id: usize) -> String{
        return String::from_iter(self.words[id].iter().map(|n| (n+97) as char));
    }
}

fn arr_eq(arr_1 : &[u8; 5], arr_2 : &[u8; 5]) -> bool{
    arr_1.iter().zip(arr_2.iter()).all(|(a,b)| a == b) 
}

//takes words from 5letterhiddenwords.txt and stores into array
fn words_to_arr(words: Vec<&str>) -> Vec<[u8; 5]> {
    let mut res: Vec<[u8; 5]> = Vec::new();
    //let mut line_number: usize = 0;
    for word in words{
        let mut hold: [u8;5] = [0;5];
        for (i, c) in word.chars().enumerate() {
            hold[i] = (c as u8) - 97;
        }
        res.push(hold);
    }
    res
}

fn u8arr_as_str(arr: &[u8; 5]) -> String{
    return String::from_iter(arr.iter().map(|n| (n+97) as char));
}