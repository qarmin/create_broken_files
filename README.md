unwrap# Create Broken Files
This app is generates broken files from valid ones.

It is useful in fuzzing parsers/apps like [image-rs](https://github.com/image-rs/image/)(loading
images), [ruff](https://github.com/charliermarsh/ruff)(parsing and linting python files)
or [godot](https://github.com/godotengine/godot)(loading models/images).

Typical fuzzer workflow:

- integrate fuzzer into component that can receive data
- run fuzzer
- wait for crash/invalid memory usage

This approach is quite good at start, because fuzzers like libfuzzer, are really fast when testing very small inputs.
With problems that happens with only bigger and specific inputs, such tools are rarely usable. From what I have seen
they usually tests such inputs - 3 spaces, 150 slashes and 20 commas with all possible combinations - which are almost
never used in programs(which of course does not mean that such bugs not happens and should not be fixed).  
But anyway, it is good to run fuzzer, before this app to find quite small reproduction projects(fuzzers are really good
at minimizing inputs that crashes)

Create Broken Files workflow:

- prepare different and valid input files(if you want to test 3d model importer then prepare files with
  extension - [fbx, dae, gltf] if image importer, then - [jpg, png, gif] etc.)
- run this app - this will create a lot of broken files
- run your script/app to import broken files(remember to run sanitizers if possible) and wait for crash

When testing `ruff` with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)(typical fuzzer build at the top
libFuzzer) after 2 hours of checking(mostly `/` characters + some random bytes) I found 1 crash and 1 error that not
crashed fuzzer(I found it in logs).

This app + script to find exact file that causes problems found in 30 minutes almost 60 errors and crashes(script
doesn't distinguish them) - some are probably duplicated, not reported them yet.

## How it works so good?

I have noticed that in the case of normal fuzzers the code coverage from a single run is very small and it is almost
never possible to test a significant portion of the code in a reasonable time.

Correct files, however, pass through most of the code without much trouble. Therefore, when parsing a file that is
slightly corrupted, its small corruption may allow you to pass through a significant portion of the code and only then
test the error handling nested deep in the code, which an ordinary fuzzer could reach after hundreds of thousands of
years.

## Modes

App contains 2 modes - binary/utf-8

In binary mode app is allowed to:

- modify values of random bytes
- split content of files in random place
- remove random bytes

In UTF-8 mode, everything works on characters, so it is by default slower, but allows to

- modify values of random characters
- split content of files in random place between characters
- remove random characters
- adds random words into text - especially useful when testing language parsers, e.g. in `ruff` which tests Python I
  added such words to increase crashes - "False" "await" "else" "import" "pass" "None" "break" "except" "in" "raise" "
  True" "class" "finally" "is" "return" "and" "continue" "for" "lambda" "try" "as" "def" "from" "nonlocal" "while" "
  assert" "del" "global" "not" "with" "async" "elif" "if" "or" "yield" "__init
  __" ":" "?" "[" "\"" "\'" "]" "}" "{" "|" "\\" ";" "_" "-" "**" "*" "/" "!"

If input file contained at start only utf-8 characters, then in this mode also should contain only them in output file.

## Usage

At start just install it with

```
cargo install create_broken_files
```

Usage:

```
create_broken_files --input-path <INPUT> --output-path <OUTPUT> --number-of-broken-files <NUMBER> [--character-mode <IS_CHARACTER_MODE>] [--special-words <WORDS>]
```

- `input-path` - input path of folder or file to use(folders are only checked with depth 1)
- `output-path` - path where generated files will be placed
- `number-of-broken-files` - number of files that will be generated from one input file(real number may be a little
  slower)
- `character-mode` - `c` - [OPTIONAL, default_value=false] if `true` change mode to utf-8(described above)
- `special-words` - `s` - [OPTIONAL, default_value=[]], works only when character-mode is true, adds random words from
  provided ones to file
- `connect_multiple_files` - `m` - [OPTIONAL, default_value=false], if `true` then some files will be added at the end
  of
  current file

Example(short version of long names are available):

```
create_broken_files -i /home/rafal/inputs -o /home/pli -n 1000
```

```
create_broken_files -i /home/rafal/Desktop/22.txt -o /home/rafal/Desktop/33 -n 10 -c -m -s "False" "await" "else"
```
