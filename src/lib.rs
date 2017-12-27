extern crate flate2;
#[macro_use]
extern crate log;
extern crate time;
extern crate vec_map;

use std::{fs, io, path};
use sat::*;
use sat::minisat::budget::Budget;

pub mod sat;


pub enum SolverOptions {
    Core(minisat::CoreSettings),
    Simp(minisat::SimpSettings),
}

pub struct MainOptions {
    pub strict: bool,
    pub pre: bool,
    pub solve: bool,
    pub in_path: path::PathBuf,
    pub out_path: Option<path::PathBuf>,
    pub dimacs_path: Option<path::PathBuf>,
}


pub fn solve(main_opts: MainOptions, solver_opts: SolverOptions) -> io::Result<()> {
    match solver_opts {
        SolverOptions::Core(opts) => {
            let solver = minisat::CoreSolver::new(opts);
            solve_with(solver, main_opts)
        }

        SolverOptions::Simp(opts) => {
            let mut solver = minisat::SimpSolver::new(opts);
            if !main_opts.pre {
                solver.preprocess(&Budget::new());
            }
            solve_with(solver, main_opts)
        }
    }
}


pub fn solve_with<S: Solver>(mut solver: S, options: MainOptions) -> io::Result<()> {
    let initial_time = time::precise_time_s();

    info!("============================[ Problem Statistics ]=============================");
    info!("|                                                                             |");

    let backward_subst = dimacs::parse_file(&options.in_path, &mut solver, options.strict)?;

    info!(
        "|  Number of variables:  {:12}                                         |",
        solver.n_vars()
    );
    info!(
        "|  Number of clauses:    {:12}                                         |",
        solver.n_clauses()
    );

    let parsed_time = time::precise_time_s();
    info!(
        "|  Parse time:           {:12.2} s                                       |",
        parsed_time - initial_time
    );

    let mut budget = Budget::new();
    budget.off();

    let elim_res = solver.preprocess(&budget);
    {
        let simplified_time = time::precise_time_s();
        info!(
            "|  Simplification time:  {:12.2} s                                       |",
            simplified_time - parsed_time
        );
    }

    info!("|                                                                             |");

    let result = if !elim_res {
        info!("===============================================================================");
        info!("Solved by simplification");
        SolveRes::UnSAT(Stats::default())
    } else {
        let result =
            if options.solve {
                solver.solve_limited(&budget, &[])
            } else {
                info!("===============================================================================");
                SolveRes::Interrupted(0.0, solver)
            };

        //            if let TotalResult::Interrupted = result {
        //                if let Some(path) = options.dimacs_path {
        //                    let mut out = try!(fs::File::create(path));
        //                    try!(dimacs::write(&mut out, &solver));
        //                }
        //            }

        result
    };

    let cpu_time = time::precise_time_s() - initial_time;
    match result {
        SolveRes::UnSAT(ref stats) => {
            print_stats(stats, cpu_time);
            println!("UNSATISFIABLE");
        }

        SolveRes::Interrupted(_, ref s) => {
            print_stats(&s.stats(), cpu_time);
            println!("INDETERMINATE");
        }

        SolveRes::SAT(ref model, ref stats) => {
            print_stats(stats, cpu_time);
            println!("SATISFIABLE");
            assert!(
                dimacs::validate_model_file(&options.in_path, &backward_subst, &model)?,
                "SELF-CHECK FAILED"
            );
        }
    }

    if let Some(path) = options.out_path {
        dimacs::write_result(fs::File::create(path)?, result, &backward_subst)?;
    }

    Ok(())
}

fn print_stats(stats: &Stats, cpu_time: f64) {
    info!("restarts              : {:<12}", stats.restarts);
    info!(
        "conflicts             : {:<12}   ({:.0} /sec)",
        stats.conflicts,
        (stats.conflicts as f64) / cpu_time
    );

    info!(
        "decisions             : {:<12}   ({:4.2} % random) ({:.0} /sec)",
        stats.decisions,
        (stats.rnd_decisions as f64) * 100.0 / (stats.decisions as f64),
        (stats.decisions as f64) / cpu_time
    );

    info!(
        "propagations          : {:<12}   ({:.0} /sec)",
        stats.propagations,
        (stats.propagations as f64) / cpu_time
    );

    info!(
        "conflict literals     : {:<12}   ({:4.2} % deleted)",
        stats.tot_literals,
        (stats.del_literals as f64) * 100.0 / ((stats.del_literals + stats.tot_literals) as f64)
    );

    info!("Memory used           : {:.2} MB", 0.0);
    info!("CPU time              : {} s", cpu_time);
    info!("");
}
