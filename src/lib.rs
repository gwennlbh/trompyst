use lambda_calculus::*;
use std::fmt::Display;
use wasm_bindgen::prelude::*;

#[cfg(test)]
use lambda_calculus::{
    combinators::{I, K, S, Y},
    data::{boolean, num::church::pred},
};

const DEBUG: bool = false;

macro_rules! debugln {
    ($($arg:tt)*) => {
        {
            if DEBUG { println!($($arg)*); }
        }
    };
}

/// Returns array of (number of abstractions the variable is under, debruijn index)
fn variable_connections(term: &Term, depth: usize) -> Vec<(usize, usize)> {
    match term {
        App(boxed) => match boxed.as_ref() {
            (lhs, rhs) => {
                let mut result = variable_connections(lhs, depth);
                result.extend(variable_connections(rhs, depth));
                result
            }
        },
        Abs(boxed) => variable_connections(boxed.as_ref(), depth + 1),
        Var(index) => vec![(depth, *index)],
    }
}

#[wasm_bindgen]
pub struct Diagram {
    cells: Vec<Vec<char>>,
}

#[wasm_bindgen]
pub fn cells_of_diagram(diagram: &Diagram) -> String {
    let mut result = String::new();
    for row in &diagram.cells {
        for cell in row {
            if *cell == ' ' {
                result.push(' ');
            } else {
                result.push('.');
            }
        }
        result.push('\n');
    }
    result
}

#[wasm_bindgen]
pub fn render_from_debrujin(expression: String) -> Diagram {
    let term = parse(&expression, DeBruijn).unwrap();
    let diagram = Diagram::from(term);
    debugln!("Final diagram: \n{}", diagram);
    diagram
}

#[wasm_bindgen]
pub fn render_from_classic(expression: String) -> Diagram {
    let term = parse(&expression, Classic).unwrap();
    let diagram = Diagram::from(term);
    debugln!("Final diagram: \n{}", diagram);
    diagram
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
    match term {
        App(boxed) => match boxed.as_ref() {
            (lhs, rhs) => max_depth(lhs).max(max_depth(rhs)),
        },
        Abs(boxed) => 1 + max_depth(boxed.as_ref()),
        Var(_) => 0,
    }
}

#[test]
fn test_max_depth() {
    let results = vec![
        (I(), 1),
        (K(), 2),
        (S(), 3),
        (Y(), 2),
        (boolean::fls(), 2),
        (2.into_church(), 2),
        (3.into_church(), 2),
        (4.into_church(), 2),
        (pred(), 5),
    ];
    for (term, expected) in results {
        let result = max_depth(&term);
        assert_eq!(
            result, expected,
            "max_depth of {:?} should be {}",
            term, expected
        );
    }
}

impl From<Term> for Diagram {
    fn from(term: Term) -> Self {
        fn render(term: Term, max_depth: usize, depth: usize) -> Diagram {
            match term {
                App(boxed) => {
                    let (lhs, rhs) = boxed.as_ref();
                    let lhs_diagram = render(lhs.clone(), max_depth, depth);
                    let rhs_diagram = render(rhs.clone(), max_depth, depth);
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
                Abs(boxed) => {
                    let body = boxed.as_ref();
                    let depth = depth + 1;
                    let body_variables = variable_connections(&body, depth);
                    let body_diagram = render(body.clone(), max_depth, depth);
                    debugln!(
                        "ABSTRACT {:?} Variables: {:?}; Depth: (real {} rev {} max {})",
                        body,
                        body_variables,
                        depth,
                        max_depth - depth,
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
                    debugln!(
                        "Variables [(wants connection to, index)]: {:?}",
                        body_variables
                            .clone()
                            .into_iter()
                            .map(|(depth, index)| (depth - index, index))
                            .collect::<Vec<_>>()
                    );
                    // Check variables that connect to this application or to another one above
                    for (i, &(depth_of_variable, index)) in body_variables.iter().enumerate() {
                        // if let Some(index) = connection {
                        //     debugln!(
                        //         "Connection: {} at {}: {} < {} ?",
                        //         index,
                        //         i,
                        //         (max_depth as isize - index as isize),
                        //         (depth as isize)
                        //     );
                        // }
                        if depth_of_variable - index <= depth - 1 {
                            let position = 1 + (i * 4);
                            result.cells[1][position] = '.';
                        }
                    }
                    debugln!("Final diagram: \n{}", result);
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
        let d = render(term.clone(), max_depth(&term), 0);
        debugln!("\n");
        d
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

#[test]
fn test_omega() {
    let term = parse("(λ11)(λ11)", DeBruijn).unwrap();
    let diagram = Diagram::from(term);
    let resultstring = diagram.to_string().replace("\u{2588}", ".");
    assert_eq!(
        resultstring,
        vec![
            "....... .......",
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
