use std::io;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;

const SUDOKU_SIZE :usize = 9;
const SUDOKU_BLOCKS :usize = 3;

fn read_input() -> [[u8; SUDOKU_SIZE]; SUDOKU_SIZE] {
    let mut sudoku = [[0 as u8; SUDOKU_SIZE]; SUDOKU_SIZE];
    for i in 0..SUDOKU_SIZE {
        let mut input = String::new();
        match io::stdin().read_line(&mut input){
            Ok(n) => if n < SUDOKU_SIZE {
                panic!("Not enough input");
            }
            Err(_) => panic!("Error reading line")
        }
        for (j, c) in (input[0..SUDOKU_SIZE]).chars().enumerate() {
            sudoku[i][j] = match c.to_string().parse(){
                Ok(n) => {n}
                Err(_) => 0
            }
        }
    }
    sudoku
}

fn prep_inputs(sudoku : &[[u8; SUDOKU_SIZE]; SUDOKU_SIZE])
               -> [[[bool; SUDOKU_SIZE]; SUDOKU_SIZE];SUDOKU_SIZE] {
    let mut input = [[[true; SUDOKU_SIZE]; SUDOKU_SIZE]; SUDOKU_SIZE];
    for (i, row) in sudoku.iter().enumerate(){
        for (j, col) in row.iter().enumerate(){
            if *col != (0 as u8) {
                input[i][j] = [false; SUDOKU_SIZE];
                input[i][j][(*col-1) as usize] = true;
            }
        }
    }
    input
}

trait ConstraintPropagation {
    fn fwd(&self, cur:&[bool]) -> (HashMap<[bool;SUDOKU_SIZE], bool>, usize);
    fn bwd(&self, cur:&[bool]) -> (HashMap<[bool;SUDOKU_SIZE], bool>, usize);
    fn out(&self, post:&(HashMap<[bool;SUDOKU_SIZE], bool>, usize)) -> [bool;SUDOKU_SIZE];
}

impl ConstraintPropagation for (HashMap<[bool;SUDOKU_SIZE], bool>, usize)
{
    fn fwd(&self, cur:&[bool])
           -> (HashMap<[bool;SUDOKU_SIZE], bool>, usize)
    {
        let mut post = HashMap::new();
        for (k,v) in self.0.iter() {
            for i in 0..SUDOKU_SIZE {
                if !k[i] && *v && cur[i] {
                    let mut l=k.clone();
                    l[i]=true;
                    if let Some(b) = &post.insert(l, *v && cur[i]) {
                        post.insert(l, (*v && cur[i]) || *b); // <- binary max product
                    }
                }
            }
        }
        (post, self.1+1)
    }
    fn bwd(&self, cur:&[bool])
           -> (HashMap<[bool;SUDOKU_SIZE], bool>, usize)
    {
        let mut pre = HashMap::new();
        for (k,v) in self.0.iter() {
            for i in 0..SUDOKU_SIZE {
                if k[i] && *v && cur[i] {
                    let mut l=k.clone();
                    l[i]=false;
                    if let Some(b) = &pre.insert(l, *v && cur[i]) {
                        pre.insert(l, (*v && cur[i]) || *b); // <- binary max product
                    }
                }
            }
        }
        (pre, self.1-1)
    }
    fn out(&self, post:&(HashMap<[bool;SUDOKU_SIZE], bool>, usize))
           -> [bool; SUDOKU_SIZE]
    {
        let mut out = [false;SUDOKU_SIZE];
        for (k,v) in self.0.iter() {
            for i in 0..SUDOKU_SIZE {
                let mut l=k.clone();
                l[i] = !l[i];
                if let Some(b) = post.0.get(&l) {
                    out[i] |= *v && *b;
                }
            }
        }
        out
    }
}

fn check_constraints(inp : &[[bool; SUDOKU_SIZE]; SUDOKU_SIZE])
                     -> [[bool; SUDOKU_SIZE]; SUDOKU_SIZE]
{
    let mut s_f : Vec<(HashMap<[bool;SUDOKU_SIZE], bool>, usize)>
        = vec![(HashMap::new(), 0); SUDOKU_SIZE+1];
    s_f[0].0.insert([false;SUDOKU_SIZE],true);
    s_f[SUDOKU_SIZE].1 = SUDOKU_SIZE;
    s_f[SUDOKU_SIZE].0.insert([true;SUDOKU_SIZE], true);
    let mut s_b = s_f.clone();
    for i in 1..SUDOKU_SIZE {
        s_f[i]=s_f[i-1].fwd(&inp[i-1])
    }

    let mut ret = [[false; SUDOKU_SIZE]; SUDOKU_SIZE];
    for i in (0..SUDOKU_SIZE).rev() {
        let out = s_f[i].out(&s_b[i+1]);
        for (c,v) in out.iter().enumerate() {
            ret[i][c] = *v && inp[i][c];
        }
        s_b[i] = s_b[i+1].bwd(&ret[i]);
    }

    /* DEBUG
    println!("-->{:?}", s_f[0]);
    println!("==^{:?}", inp[0]);
    println!("-->{:?}", s_f[1]);
    println!("==^{:?}", inp[1]);
    println!("-->{:?}", s_f[2]);
    println!("==^{:?}", inp[2]);
    println!("-->{:?}", s_f[3]);
    println!("==^{:?}", inp[3]);
    println!("-->{:?}", s_f[4]);
    println!("---------------");
    println!("<--{:?}", s_b[4]);
    println!("==v{:?}", ret[3]);
    println!("<--{:?}", s_b[3]);
    println!("==v{:?}", ret[2]);
    println!("<--{:?}", s_b[2]);
    println!("==v{:?}", ret[1]);
    println!("<--{:?}", s_b[1]);
    println!("==v{:?}", ret[0]);
    println!("<--{:?}", s_b[0]);
     */

    ret
}

fn main() {
    let mut sudoku = read_input();
    let mut inputs_f = prep_inputs(&sudoku);
    let mut ambig = 0;
    let mut count = 0;

    loop {
        count += 1;
        println!("{}-------------------------",count);
        for i in  0..SUDOKU_SIZE {
            println!("{:?}", sudoku[i]);
        }
        println!("---------------------------");

        let mut inputs_row = [[[false; SUDOKU_SIZE]; SUDOKU_SIZE];SUDOKU_SIZE];
        let mut inputs_col = [[[false; SUDOKU_SIZE]; SUDOKU_SIZE];SUDOKU_SIZE];
        let mut inputs_block = [[[false; SUDOKU_SIZE]; SUDOKU_SIZE];SUDOKU_SIZE];

        // row
        for i in  0..SUDOKU_SIZE {
            inputs_row[i] = check_constraints(&inputs_f[i]);
        }

        // col
        for i in  0..SUDOKU_SIZE {
            let mut inp = [[false; SUDOKU_SIZE]; SUDOKU_SIZE];
            for (c,_v) in inputs_f.iter().enumerate() {
                inp[c] = inputs_f[c][i];
            }
            let inp_col = check_constraints(&inp);
            for (c,v) in inp_col.iter().enumerate() {
                inputs_col[c][i] = *v;
            }
        }

        // block
        for i in (0..SUDOKU_SIZE).collect::<Vec<usize>>().chunks(SUDOKU_BLOCKS){
            for j in (0..SUDOKU_SIZE).collect::<Vec<usize>>().chunks(SUDOKU_BLOCKS){
                let mut inp = [[false; SUDOKU_SIZE]; SUDOKU_SIZE];
                let mut c = 0;
                for k in i {
                    for l in j {
                        inp[c] = inputs_f[*k][*l];
                        c+=1;
                    }
                }
                let inp_blk = check_constraints(&inp);
                let mut c = 0;
                for k in i {
                    for l in j {
                        inputs_block[*k][*l] = inp_blk[c];
                        c+=1;
                    }
                }
            }
        }
        for i in  0..SUDOKU_SIZE {
            for j in  0..SUDOKU_SIZE {
                for k in  0..SUDOKU_SIZE {
                    inputs_f[i][j][k] = inputs_block[i][j][k] && inputs_col[i][j][k] && inputs_row[i][j][k];
                }
            }
        }
        let mut curambig = 0;
        for i in  0..SUDOKU_SIZE {
            for j in  0..SUDOKU_SIZE {
                let nums = inputs_f[i][j].iter().enumerate()
                    .filter(|(_c,v)|**v)
                    .map(|x|x.0+1).collect::<Vec<usize>>();
                match nums.len().cmp(&1) {
                    Ordering::Less => {panic!("Unsolvable")}
                    Ordering::Equal => {sudoku[i][j] = u8::try_from(nums[0]).expect("Internal error")}
                    Ordering::Greater => {curambig += 1;}
                }
            }
        }
        if curambig == 0 || curambig == ambig {
            break;
        }
        ambig = curambig;
    }
    for i in  0..SUDOKU_SIZE {
        println!("{:?}", sudoku[i]);
    }

}
