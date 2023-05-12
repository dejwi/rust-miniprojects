# Github web scrapper
Scrapes data from a github profile

## Run
```shell
cargo run -p gh-web-scraper 
```

## About
- gets display name, followers + follows count
- scrapes profiles about (README.md) for an email and linkedin

## Example
<img width="476" alt="image" src="https://github.com/dejwi/rust-miniprojects/assets/80927085/1c9868e3-286a-4493-b08d-91fe237b2783">


## Crates used
- reqwest
- scraper
- tokio
- linkify
- prettytable-rs
