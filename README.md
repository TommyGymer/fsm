# FSM Simulator

> A simple Rust CLI programme for simulating finite state machines

## Usage

```sh
cargo run -- -f <fsm definition file> -i <inut string>
```

An FSM can be defined as follows, with the three sections required.
There must be a transition for every combination of start state and input character.

```fsm
states:
  A
  B
  final: C

transitions:
  0: A -> B
  0: B -> C
  0: C -> A
  1: B -> A
  1: C -> B
  1: A -> C

start: A
```
