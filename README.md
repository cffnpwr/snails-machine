# Snail's Machine

## これはなに

- Turing Machine Simulatorです
- json/toml/yaml形式で状態遷移関数を記述し、それを読み込んでシミュレーションを行います

## Dependencies

- Rust

## Build

### Native Build

```sh
cargo build --release
```

### Cross Build

#### 対応ターゲット

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aaarch64-apple-darwin`

#### Build

```sh
rustup target add [TARGET]
cargo build --release --target [TARGET]
```

## Usage

```sh
./target/release/snails-machine [OPTIONS] <TAPE>
```

- TAPE: テープの初期状態を指定します
- OPTIONS:
  - `-f`, `--file`: 状態遷移関数を記述したファイルを指定します (デフォルト: `./machine.toml`)

## Example

チューリングマシンの例として10進数の加算器(Incrementer)を`machine.toml`に用意しています。

```sh
./target/release/snails-machine -f ./machine.toml 999
```

```
Running: [__999_]: (q0, 9) -> (q0, 9)
Running: [__999_]: (q0, 9) -> (q0, 9)
Running: [__999_]: (q0, 9) -> (q0, 9)
Running: [__999_]: (q0, _) -> (q1, _)
Running: [__999_]: (q1, 9) -> (q1, 0)
Running: [__990_]: (q1, 9) -> (q1, 0)
Running: [__900_]: (q1, 9) -> (q1, 0)
Running: [__000_]: (q1, _) -> (q2, 1)
 Accept: [_1000_]
```

## License

[MIT](LICENSE)
