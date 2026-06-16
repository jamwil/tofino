use httpmock::prelude::*;
use tofino::cli;

#[test]
fn test_too_few_arguments() {
    let args = vec![String::from("tofino")];
    let result = cli::cli(args);
    assert!(result.is_err_and(|e| e.to_string() == "Tofino accepts a single URL argument"));
}

#[test]
fn test_too_many_arguments() {
    let args = vec![
        String::from("tofino"),
        String::from("is"),
        String::from("grand"),
    ];
    let result = cli::cli(args);
    assert!(result.is_err_and(|e| e.to_string() == "Tofino accepts a single URL argument"));
}

#[test]
fn test_just_right_arguments() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method("GET").path("/");
        then.status(200)
            .header("Content-Type", "text/html; charset=UTF-8")
            .body("hola");
    });

    let args = vec![String::from("tofino"), String::from(&server.url("/"))];
    let result = cli::cli(args);

    mock.assert();
    assert!(result.is_ok_and(|r| r == "hola"));
}
