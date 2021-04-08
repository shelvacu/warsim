#![feature(array_map)]
use std::ops::{Range, Index};

use arr_macro::arr;

use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

type Card = u8;

const NUM_PLAYERS:usize = 2;

fn main() {
    dbg!(compete(HighestFirst{}, HighestFirst{}, 1));
    dbg!(compete(HighestFirst{}, LowestFirst{}, 1));
    dbg!(compete(Intersperse(HighestFirst{},LowestFirst{}), HighestFirst{}, 1));
    dbg!(compete(Intersperse(HighestFirst{},LowestFirst{}), LowestFirst{}, 1));
    dbg!(compete(Intersperse(LowestFirst{},HighestFirst{}), HighestFirst{}, 1));
    dbg!(compete(Intersperse(LowestFirst{},HighestFirst{}), LowestFirst{}, 1));
    dbg!(compete(Intersperse(LowestFirst{},HighestFirst{}), Intersperse(HighestFirst{},LowestFirst{}), 1));
    println!("-------");
    let rounds = 100000;
    dbg!(compete(Intersperse(HighestFirst{},LowestFirst{}), Random{}, rounds));
    dbg!(compete(Intersperse(LowestFirst{},HighestFirst{}), Random{}, rounds));
    dbg!(compete(HighestFirst{}, Random{}, rounds));
    dbg!(compete(LowestFirst{}, Random{}, rounds));
    dbg!(compete(Random{}, Random{}, rounds));
}

#[derive(Debug,Clone)]
struct GameCounts {
    pub wins:usize,
    pub ties:usize,
    pub loss:usize,
}

fn compete<S1: Strategy, S2: Strategy>(player:S1, against:S2, game_count:usize) -> GameCounts {
    let mut rng = thread_rng();
    let mut res = GameCounts{wins:0,ties:0,loss:0};
    for _ in 0..game_count {
        let mut player_deck  = vec![1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13];
        let mut against_deck = vec![1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13];
        player.order_cards(&mut player_deck[..], &mut rng);
        against.order_cards(&mut against_deck[..], &mut rng);
        let mut game = Game::new([player_deck,against_deck]);
        let mut steps:usize = 0;
        loop {
            match game.step() {
                GameState::Continue => (),
                GameState::Tie => {
                    res.ties += 1;
                    break;
                },
                GameState::Finish(i) => {
                    if i > 1 { panic!(); }
                    if i == 1 {
                        res.wins += 1;
                    } else {
                        res.loss += 1;
                    }
                    break;
                }
            }
            steps += 1;
            if steps > 100000 {
                panic!("Max step count exceeded");
            }
        }
    }
    res
}

trait Strategy {
    fn order_cards<R: Rng>(&self, cards: &mut [Card], rng: &mut R);
}

#[derive(Debug,Copy,Clone)]
struct Intersperse<S1: Strategy, S2: Strategy>(pub S1,pub S2);

impl<S1: Strategy, S2: Strategy> Strategy for Intersperse<S1, S2> {
    fn order_cards<R: Rng>(&self, cards: &mut [Card], rng: &mut R) {
        let mut set1:Vec<Card> = cards.iter().step_by(2).map(|c| *c).collect();
        let mut set2:Vec<Card> = cards.iter().skip(1).step_by(2).map(|c| *c).collect();
        self.0.order_cards(&mut set1[..], rng);
        self.1.order_cards(&mut set2[..], rng);
        for i in 0..cards.len() {
            let div = i/2;
            let rem = i%2;
            if rem == 0 {
                cards[i] = set1[div];
            } else {
                cards[i] = set2[div];
            }
        }
    }
}

#[derive(Debug,Copy,Clone)]
struct LowestFirst {}

impl Strategy for LowestFirst {
    fn order_cards<R: Rng>(&self, cards: &mut [Card], _rng: &mut R) {
        cards.sort_unstable();
    }
}

#[derive(Debug,Copy,Clone)]
struct HighestFirst {}

impl Strategy for HighestFirst {
    fn order_cards<R: Rng>(&self, cards: &mut [Card], _rng: &mut R) {
        cards.sort_unstable_by(|a,b| a.cmp(b).reverse());
    }
}

#[derive(Debug,Copy,Clone)]
struct Random {}

impl Strategy for Random {
    fn order_cards<R: Rng>(&self, cards: &mut [Card], rng: &mut R) {
        cards.shuffle(rng);
    }
}

#[derive(Debug,Clone)]
struct HistoryQueue<T> {
    history:Vec<T>,
    len:usize,
}

impl<T> HistoryQueue<T> {
    pub fn history(&self) -> &[T] {
        self.history.as_ref()
    }

    pub fn push(&mut self, value: T) {
        self.history.push(value);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(&self.history[self.history.len() - self.len - 1])
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn current_range(&self) -> Range<usize> {
        self.start()..self.history.len()
    }

    fn start(&self) -> usize {
        self.history.len() - self.len
    }
}

impl<T> Index<usize> for HistoryQueue<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        &self.history[self.start() + idx]
    }
}

impl<T> From<Vec<T>> for HistoryQueue<T> {
    fn from(vec: Vec<T>) -> HistoryQueue<T> {
        let len = vec.len();
        HistoryQueue{
            history: vec,
            len,
        }
    }
}

struct Game {
    pub hands:[HistoryQueue<Card>; NUM_PLAYERS],
    pub hands_histories:[Vec<Range<usize>>; NUM_PLAYERS]
}

enum GameState {
    Continue,
    Tie,
    Finish(usize),
}

impl Game {
    pub fn new(starting_hands: [Vec<Card>; NUM_PLAYERS]) -> Self {
        let hands = starting_hands.map(|v:Vec<Card>| v.into());
        let hands_histories = hands.clone().map(|h:HistoryQueue<Card>| vec![h.current_range()]);
        Game{
            hands,
            hands_histories,
        }
    }

    pub fn step(&mut self) -> GameState {
        // table[i].0 is whether that player is "still in" -- tied every round so far and hasn't run out of cards
        let mut table:[(bool, Vec<Card>); NUM_PLAYERS] = arr![(true, Vec::new()); 2];
        loop {
            for i in 0..NUM_PLAYERS {
                if !table[i].0 { continue; }
                if let Some(card) = self.hands[i].pop() {
                    table[i].1.push(*card)
                } else {
                    table[i].0 = false;
                }
            }

            let mut competing_cards = Vec::new();
            
            for i in 0..NUM_PLAYERS {
                if !table[i].0 { continue; }
                competing_cards.push((i, *table[i].1.last().unwrap()));
            }

            if competing_cards.is_empty() {
                return GameState::Tie;
            }

            competing_cards.sort_unstable_by_key(|(a,b)| *b);

            let winning_card = competing_cards.last().unwrap().1;

            let mut winners:Vec<usize> = competing_cards
                .iter()
                .rev()
                .take_while(|(i,c)| *c==winning_card)
                .map(|(i,c)| *i)
                .collect();
            
            if winners.len() == 1 {
                let winner = winners[0];
                let winner_hand = &mut self.hands[winner];
                let mut i = winner + 1;
                loop {
                    if i == NUM_PLAYERS { i = 0; }
                    for c in &table[i].1 {
                        winner_hand.push(*c);
                    }
                    if i == winner { break; }
                    i += 1;
                }
                break;
            } else {
                //sacrificial cards
                
                for i in 0..NUM_PLAYERS {
                    if !table[i].0 { continue; }
                    if let Some(card) = self.hands[i].pop() {
                        table[i].1.push(*card)
                    } else {
                        table[i].0 = false;
                    }
                }
            }
        }

        for i in 0..NUM_PLAYERS {
            self.hands_histories[i].push(self.hands[i].current_range());
        }

        let players_in:Vec<_> = self.hands.iter().enumerate().filter(|(i,h)| h.len() > 0).map(|(i,h)| i).collect();
        if players_in.len() > 1 {
            return GameState::Continue;
        } else {
            return GameState::Finish(players_in[0]);
        }
    }
}