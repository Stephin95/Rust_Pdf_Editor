use pdf_editor::pdf_backend::{load_pdfs,merge_pdfs};
use std::env;
use log::{debug, error, log_enabled, info, Level};
// use env_logger;



fn main() {
    let key = "RUST_LOG";
    env::set_var(key, "info ./main");
    assert_eq!(env::var(key), Ok("info ./main".to_string()));

    // let logger_level = "RUST_LOG";
    
    //To enable logging use this env variable
    // RUST_LOG="info" ./main
    // $RUST_LOG=info ./main

    if log_enabled!(Level::Info) {
        info!("Log enabled:");
    }
    debug!("log debug enabled");
    info!("Log info enabled");
    error!("Error logs enabled");
    // debug!("[root] debug");
    get_arguments()

    // println!("{:?}", args);
}

fn help() {
    println!("  Help Page\npdf_editor --merge <inputfile1> <inputfile2>.. --out <output_location>");
}

fn get_arguments() 
{
    // let args = env::args();
    let args: Vec<String> = env::args().collect();
    info!("{:?}", args);
    let args_size = args.len();
    // let min_size=usize::try_from(2).unwrap();
    if args_size == 1 
    {
        println!("No arguments given\nTry pdfEditor --help");
    } 
    else if args_size == 2 
    {
        if "--help" == &args[1] 
        {
            help();
        }
    } 
    else if args_size >= 3 
    {
        if "--merge" == &args[1] && args_size >= 4 
        {
            info!("merge command called");
            let filepath = &args;
            let mut documents=load_pdfs(filepath);
            merge_pdfs(documents)
            
        } 
        else 
        {
            println!("Not enough values for the operation supplied or unknown argument")
        }
    } 

}
