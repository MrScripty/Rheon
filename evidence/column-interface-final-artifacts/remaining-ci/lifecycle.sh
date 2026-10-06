cargo build --locked --release --bin rheon
"$RUNNER_TEMP/rheon-tests/bin/python" -m unittest discover -s tools -p 'test_*.py' -v
