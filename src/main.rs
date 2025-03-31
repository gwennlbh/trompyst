#![feature(box_patterns)]

use lambda_calculus::combinators::{I, S, Y};
use lambda_calculus::data::num::church::{fac, pred};
use lambda_calculus::{combinators::K, data::boolean, *};
use std::fmt::{Display, Write};
use std::vec;

const DEBUG: bool = true;

macro_rules! debugln {
    ($($arg:tt)*) => {
        {
            if DEBUG { println!($($arg)*); }
        }
    };
}

fn main() {
    // let term: Term = parse("λλ21(λλ1)", DeBruijn).expect("Failed to parse term");
    // if term.has_free_variables() {
    //     panic!("Term has free variables");
    // }
    // println!("Rendering {:?}", term);
    // let diagram: Diagram = term.into();
    // println!("Width: {}, Height: {}", diagram.width(), diagram.height());
    // println!("{}", diagram);
    // println!("{}", Diagram::from(4.into_church()))
    // println!("{}", Diagram::from())
    let terms = vec![
        // I(),
        // K(),
        // boolean::fls(),
        // S(),
        // Y(),
        // parse("λf.(λx.x x)(λx.f(x x))", Classic).unwrap(),
        // 2.into_church(),
        // 3.into_church(),
        // 4.into_church(),
        pred(),
        // parse("λn.λf.n(λf.λn.n(f(λf.λx.n f(f x))))(λx.f)(λx.x)", Classic).unwrap(),
        // parse("(λ11)(λ11)", DeBruijn).unwrap(),
    ];
    for term in terms {
        println!("{0} = {0:?}", term);
        println!("{}", Diagram::from(term))
    }
}

fn variable_connections(term: &Term) -> Vec<Option<usize>> {
    fn variables_dont_pierce(term: &Term, depth: usize, max_depth: usize) -> Vec<Option<usize>> {
        match term {
            App(box (lhs, rhs)) => {
                let mut vars = variables_dont_pierce(lhs, depth, max_depth);
                vars.extend(variables_dont_pierce(rhs, depth, max_depth));
                vars
            }
            Abs(box body) => variable_connections(body)
                .iter()
                .map(|maybe_var| match maybe_var {
                    None => None,
                    // Let vars pierce thru abstraction if they are bound to an abstraction above
                    Some(var) if *var > max_depth + 1 => Some(*var),
                    Some(_) => None,
                })
                .collect(),
            Var(index) => vec![Some(*index)],
        }
    }
    match term {
        Abs(box body) => variable_connections(body),
        _ => variables_dont_pierce(term, 0, max_depth(term)),
    }
}

struct Diagram {
    cells: Vec<Vec<char>>,
}

impl Display for Diagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.cells {
            for cell in row {
                if *cell == ' ' {
                    write!(f, " ")?;
                } else {
                    write!(f, "\u{2588}")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn max_depth(term: &Term) -> usize {
    fn get_max_depth(term: &Term, depth: usize) -> usize {
        debugln!("get_max_depth({:?}, {})", term, depth);
        match term {
            App(box (lhs, rhs)) => get_max_depth(lhs, depth).max(get_max_depth(rhs, depth)),
            Abs(box body) => get_max_depth(body, depth + 1),
            Var(_) => depth,
        }
    }
    get_max_depth(term, 0)
}

impl From<Term> for Diagram {
    fn from(term: Term) -> Self {
        fn render(term: Term, max_depth: usize, depth: usize) -> Diagram {
            match term {
                App(box (lhs, rhs)) => {
                    let lhs_diagram = Diagram::from(lhs.clone());
                    let rhs_diagram = Diagram::from(rhs.clone());
                    debugln!("{:?} MERGE {:?}", lhs, rhs);
                    debugln!("LHS Diagram:\n{}", lhs_diagram);
                    debugln!("RHS Diagram:\n{}", rhs_diagram);
                    let mut result = Diagram::new(
                        lhs_diagram.width() + rhs_diagram.width() + 1,
                        lhs_diagram.height().max(rhs_diagram.height()) + 2,
                    );
                    let final_height = result.height();
                    // Put lhs and rhs diagrams side by side, separated by one-pixel space
                    for (i, row) in lhs_diagram.cells.iter().enumerate() {
                        result.cells[i][..lhs_diagram.width()].copy_from_slice(row);
                    }
                    for (i, row) in rhs_diagram.cells.iter().enumerate() {
                        result.cells[i][lhs_diagram.width() + 1..].copy_from_slice(row);
                    }
                    // Connect connector from LHS to result's connector
                    let lhs_connector_y = lhs_diagram.height() - 1;
                    for y in lhs_connector_y..(final_height - 1) {
                        result.cells[y][1] = '.';
                    }
                    // Connect connector from RHS to result's connector
                    let rhs_connector_y = rhs_diagram.height() - 1;
                    for y in rhs_connector_y..(final_height - 1) {
                        result.cells[y][lhs_diagram.width() + 2] = '.';
                    }
                    // Draw line at one pixel from bottom, on every column except the first one
                    for i in 1..(result.width() - rhs_diagram.width() + 2) {
                        result.cells[final_height - 2][i] = '.';
                    }

                    // Draw connection point
                    result.cells[final_height - 1][1] = '.';
                    debugln!("Result diagram: \n{}", result);
                    debugln!("\n");
                    result
                }
                Abs(box body) => {
                    let depth = depth + 1;
                    debugln!("{} - {}", max_depth, depth);
                    let reverse_depth = max_depth - depth;
                    let body_variables = variable_connections(&body);
                    let body_diagram = render(body.clone(), max_depth, depth);
                    debugln!(
                        "ABSTRACT {:?} Variables: {:?}; Depth: {} (rev {}, max {})",
                        body,
                        body_variables,
                        depth,
                        reverse_depth,
                        max_depth
                    );
                    debugln!("Body diagram: \n{}", body_diagram);
                    let mut result = Diagram::new(body_diagram.width(), body_diagram.height() + 2);
                    // Add line
                    for i in 0..result.width() {
                        result.cells[0][i] = '.';
                    }
                    // Add body content below, after a line of margin
                    for (i, row) in body_diagram.cells.iter().enumerate() {
                        result.cells[i + 2][..body_diagram.width()].copy_from_slice(row);
                    }
                    // Check variables that connect to this application or to another one above
                    for (i, &connection) in body_variables.iter().enumerate() {
                        // if let Some(index) = connection {
                        //     debugln!(
                        //         "Connection: {} at {}: {} < {} ?",
                        //         index,
                        //         i,
                        //         (max_depth as isize - index as isize),
                        //         (depth as isize)
                        //     );
                        // }
                        match connection {
                            Some(index) if index > reverse_depth => {
                                let position = 1 + (i * 4);
                                result.cells[1][position] = '.';
                            }
                            _ => (),
                        }
                    }
                    debugln!("\n");
                    result
                }
                Var(_) => {
                    // Draw connection point
                    Diagram {
                        cells: vec![vec![' ', '.', ' '], vec![' ', '.', ' ']],
                    }
                }
            }
        }
        render(term.clone(), max_depth(&term), 0)
    }
}

impl Diagram {
    fn width(&self) -> usize {
        self.cells.iter().map(|row| row.len()).max().unwrap_or(0)
    }

    fn height(&self) -> usize {
        self.cells.len()
    }

    fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![' '; width]; height];
        Diagram { cells }
    }
}

#[test]
fn test_identity() {
    let term = I();
    let diagram = Diagram::from(term.clone());
    assert_eq!(diagram.width(), 3);
    assert_eq!(diagram.height(), 4);
    assert_eq!(diagram.cells[0], vec!['.', '.', '.']);
    assert_eq!(diagram.cells[1], vec![' ', '.', ' ']);
    assert_eq!(diagram.cells[2], vec![' ', '.', ' ']);
    assert_eq!(diagram.cells[3], vec![' ', '.', ' ']);
}

#[test]
fn test_k() {
    let term = K();
    let diagram = Diagram::from(term.clone());
    assert_eq!(diagram.width(), 3);
    assert_eq!(diagram.height(), 6);
    assert_eq!(diagram.cells[0], vec!['.', '.', '.']);
    assert_eq!(diagram.cells[1], vec![' ', '.', ' ']);
    assert_eq!(diagram.cells[2], vec!['.', '.', '.']);
    assert_eq!(diagram.cells[3], vec![' ', '.', ' ']);
    assert_eq!(diagram.cells[4], vec![' ', '.', ' ']);
    assert_eq!(diagram.cells[5], vec![' ', '.', ' ']);
}

#[test]
fn test_false() {
    let term = boolean::fls();
    let diagram = Diagram::from(term.clone());
    assert_eq!(diagram.width(), 3);
    assert_eq!(diagram.height(), 6);
    assert_eq!(diagram.cells[0], vec!['.', '.', '.']);
    assert_eq!(diagram.cells[1], vec![' ', ' ', ' ']);
    assert_eq!(diagram.cells[2], vec!['.', '.', '.']);
    assert_eq!(diagram.cells[3], vec![' ', '.', ' ']);
    assert_eq!(diagram.cells[4], vec![' ', '.', ' ']);
    assert_eq!(diagram.cells[5], vec![' ', '.', ' ']);
}

#[test]
fn test_s() {
    let term = S();
    let diagram = Diagram::from(term.clone());
    let resultstring = diagram.to_string().replace("\u{2588}", ".");
    assert_eq!(
        resultstring,
        vec![
            "...............",
            " .             ",
            "...............",
            " .       .     ",
            "...............",
            " .   .   .   . ",
            " .   .   .   . ",
            " .   .   .   . ",
            " .....   ..... ",
            " .       .     ",
            " .........     ",
            " .             ",
        ]
        .join("\n")
            + "\n"
    );
}

#[test]
fn test_2() {
    let term = 2.into_church();
    let diagram = Diagram::from(term);
    let resultstring = diagram.to_string().replace("\u{2588}", ".");
    assert_eq!(
        resultstring,
        vec![
            "...........",
            " .   .     ",
            "...........",
            " .   .   . ",
            " .   .   . ",
            " .   .   . ",
            " .   ..... ",
            " .   .     ",
            " .....     ",
            " .         ",
        ]
        .join("\n")
            + "\n"
    );
}

#[test]
fn test_3() {
    let term = 3.into_church();
    let diagram = Diagram::from(term);
    let resultstring = diagram.to_string().replace("\u{2588}", ".");
    assert_eq!(
        resultstring,
        vec![
            "...............",
            " .   .   .     ",
            "...............",
            " .   .   .   . ",
            " .   .   .   . ",
            " .   .   .   . ",
            " .   .   ..... ",
            " .   .   .     ",
            " .   .....     ",
            " .   .         ",
            " .....         ",
            " .             ",
        ]
        .join("\n")
            + "\n"
    );
}
