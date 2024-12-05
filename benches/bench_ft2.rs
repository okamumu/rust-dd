// use dd::common::*;
use dd::nodes::*;
use dd::bdd_mut::*;
use dd::dot::*;
use dd::gc::*;

use std::io::BufWriter;

type Bdd = BddMut;
type Node = BddMutNode;

fn clock<F>(s: &str, f: F) where F: FnOnce() {
    let start = std::time::Instant::now();
    f();
    let end = start.elapsed();
    println!("{}: time {}", s, end.as_secs_f64());
}

// fn dot_print(f: &Node) {
//     let mut buf = vec![];
//     {
//         let mut io = BufWriter::new(&mut buf);
//         f.dot(&mut io);
//     }
//     let s = std::str::from_utf8(&buf).unwrap();
//     println!("{}", s);
// }

fn find_minpath(dd: &Bdd, f: &Node, current: usize, minimum: &mut usize,
    path: &mut [usize], pset: &mut Vec<Box<[usize]>>) {
    match f {
        Node::One if current < *minimum => {
            *minimum = current;
            pset.clear();
            pset.push(path.to_vec().into_boxed_slice());
        },
        Node::One if current == *minimum => {
            pset.push(path.to_vec().into_boxed_slice());
        },
        Node::NonTerminal(fnode) if current <= *minimum => {
            for (i,x) in fnode.borrow().iter().enumerate() {
                path[fnode.borrow().level()] = i;
                find_minpath(dd, x, current+i, minimum, path, pset);
                path[fnode.borrow().level()] = 0;
            }
        },
        _ => (),
    }
}

fn make_bdd_from_pathset(dd: &mut Bdd, vars: &[Node], pset: &[Box<[usize]>]) -> Node {
    let mut f = dd.zero();
    for a in pset.into_iter() {
        let mut tmp = dd.one();
        for (i,b) in a.into_iter().enumerate() {
            if *b == 1 {
                tmp = dd.and(&tmp, &vars[i]);
            }
        }
        f = dd.or(&f, &tmp);
    }
    f
}

fn mcs(dd: &mut Bdd, vars: &[Node], f: &Node) -> Vec<Box<[usize]>> {
    let mut result: Vec<Box<[usize]>> = Vec::new();
    let mut r = f.clone();
    let mut path = vec![0; vars.len()];
    let mut pset: Vec<Box<[usize]>> = Vec::new();
    while r != Node::Zero {
        let mut minimum = usize::MAX;
        find_minpath(dd, &r, 0, &mut minimum, &mut path, &mut pset);
        let g = make_bdd_from_pathset(dd, &vars, &pset);
        result.append(&mut pset);
        r = dd.imp(&r, &g);
        r = dd.not(&r);
    }
    result
}

fn make_ft(dd: &mut Bdd, vars: &[Node], x: &[Vec<usize>]) -> Node {
    let mut f = dd.zero();
    for a in x.into_iter() {
        let mut tmp = dd.one();
        for b in a.into_iter() {
            tmp = dd.and(&tmp, &vars[*b]);
        }
        f = dd.or(&f, &tmp);
    }
    f
}

fn bench_ft1 () {
    let mut dd: Bdd = Bdd::new();
    let labels = [0,1,2,3,4,5,6,7];
    let headers = labels.into_iter().map(|i| dd.header(i, &format!("{}", i))).collect::<Vec<_>>();
    let vars = labels.into_iter().map(|i| dd.create_node(&headers[i], &dd.zero(), &dd.one())).collect::<Vec<_>>();

    let n = vec![
        vec![2],
        vec![4,0,7],
        vec![3,5,2],
        vec![1,7,3],
        vec![5,7,3],
        vec![1,2,6],
    ];

    let f = make_ft(&mut dd, &vars, &n);
    println!("size {:?}", dd.size());

    let result = mcs(&mut dd, &vars, &f);
    println!("{:?}", result);
    println!("size {:?}", dd.size());

    dd.gc(&[f.clone()]);
    println!("size {:?}", dd.size());

    let mut buf = vec![];
    {
        let mut io = BufWriter::new(&mut buf);
        f.dot(&mut io);
    }
    let s = std::str::from_utf8(&buf).unwrap();
    println!("{}", s);
}

fn linrange(x0: f64, x1: f64, n: usize) -> Vec<f64> {
	let mut ans = vec![0.0; n];
	let mut x = x0;
	let d = (x1 - x0) / ((n - 1) as f64);
	for a in ans.iter_mut() {
		*a = x;
		x += d;
	}
	ans
}

fn make_test_data() -> (Vec<usize>, Vec<Vec<usize>>) {
    let data = vec![
		vec![0.2799439284864673, 0.019039179685146124],
		vec![0.17006659269016278, 0.26812585079180584],
		vec![0.37160535186424815, 0.28336464179809084],
		vec![0.39279646612146735, 0.2789501222723816],
		vec![0.44911286867346534, 0.2067605663915406],
		vec![0.505207192733002, 0.07778618522601977],
		vec![0.381127966318632, 0.36580119057192695],
		vec![0.14314120834324617, 0.5282542334011777],
		vec![0.2236688207291726, 0.5027191237033151],
		vec![0.5865981007905481, 0.05016684053503706],
		vec![0.2157117712338983, 0.5699545901561343],
		vec![0.6600618683347792, 0.006513992462842788],
		vec![0.6964756269215944, 0.031164261499776913],
		vec![0.5572474263734104, 0.5457354609512821],
		vec![0.38370575109517757, 0.6870518034076929],
		vec![0.14047278702240318, 0.8099471630562083],
		vec![0.6117903795750386, 0.6200985639530681],
		vec![0.8350140149860443, 0.26002375370524433],
		vec![0.621745085645081, 0.6249760808944675],
		vec![0.9223171788742697, 0.040441694559945285],
		vec![0.40157225733130186, 0.8622123559544623],
		vec![0.5654235033016655, 0.7840149804945578],
		vec![0.8605048496383341, 0.48642029259985065],
		vec![0.5627138851056968, 0.8499394786290626],
		vec![0.7124617313668333, 0.7347698978106127],
		vec![0.9656307414336753, 0.3647058735973785],
		vec![0.9944967296698335, 0.548297306757731],
		vec![0.5733819926662398, 0.9813641372820436],
		vec![0.9236020954856745, 0.7540471034450749],
		vec![0.8910887808888235, 0.8901974734237881],
    ];
	let labels = [13, 19, 2, 14, 29, 23, 3, 26, 25, 7, 9, 27, 12, 30, 17, 24, 8, 4, 18, 5, 20, 21, 28, 1, 16, 10, 15, 6, 11, 22];
    let r = 0.3;
    let gridn = 1000;
	let ddx = linrange(0.0, 1.0, gridn);
	let ddy = linrange(0.0, 1.0, gridn);
	let mut result: Vec<Vec<usize>> = Vec::new();
	for x in &ddx {
		for y in &ddy {
			let mut v = Vec::new();
			for (i, p) in data.iter().enumerate() {
				let tmpx = x - p[0];
                let tmpy = y - p[1];
				if tmpx*tmpx+tmpy*tmpy < r*r {
					v.push(labels[i]-1)
				}
			}
			result.push(v)
		}
	}
	(labels.to_vec(), result)
}

fn bench_ft2() {
    let (labels, data) = make_test_data();

    let mut dd: Bdd = Bdd::new();
    let headers = (0..labels.len()).into_iter().map(|i| dd.header(i, &format!("{}", labels[i]))).collect::<Vec<_>>();
    let vars = (0..labels.len()).into_iter().map(|i| dd.create_node(&headers[i], &dd.zero(), &dd.one())).collect::<Vec<_>>();

    let f = make_ft(&mut dd, &vars, &data);
    println!("size {:?}", dd.size());

    let result = mcs(&mut dd, &vars, &f);
    println!("{:?}", result);
    println!("size {:?}", dd.size());

    dd.gc(&[f.clone()]);
    println!("size {:?}", dd.size());
}

fn main() {
    clock("bench ft1", bench_ft1);
    clock("bench ft2", bench_ft2);
}