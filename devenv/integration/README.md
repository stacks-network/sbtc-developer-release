## Containerized integration testing

Hi! Congratulations on making it this far. By now, you should know how to launch
a full node for local development using devnev. To take a step further, these
are the steps you need to follow to run and implement integration tests using
devenv.

## Quickstart

1. Build the testbed image with the source and binaries to execute the
   integration tests.
2. Run the testing script.

```bash
> devenv$ cd integration
> devenv/integration$ ./bin/build
> cd -
> devenv$ ./integration/bin/test
```

You will see lines like the one below if your tests are completed successfully.

```
Runner: 71c1beca7261e2c1b706fa0e9eeb3823ad56977c4846c89b25a315a48a1bbda5 exited with err_no: 0
Runner: fb2da5362115b5ae37874d39831bbf249b2928c58296dfb23af94b7083d8455b exited with err_no: 0
```

Use the container id to inspect the runner for more details.

```
> devenv$ ./logs.sh 71c1beca7261e2c1b706fa0e9eeb3823ad56977c4846c89b25a315a48a1bbda5
```

The script will abort the moment a container fails. The script will print the
logs from the first failed container. You must stop the nodes with
`docker stop $(docker ps -q)`. You can also rerun `test` to 'down' and 'up' any
dangling container and re-execute tests once you have fixed and rebuilt the
testbed image.

## QuickStart

1. Build the testbed image that has the source and binaries to execute the integration tests.
2. Run the testing script.

```bash
> devenv$ cd integration
> devenv/integration$ ./bin/build
> cd -
> devenv$ ./integration/bin/test
```

You will see lines like below if your tests completed succesfully.

```
Runner: 71c1beca7261e2c1b706fa0e9eeb3823ad56977c4846c89b25a315a48a1bbda5 exited with err_no: 0
Runner: fb2da5362115b5ae37874d39831bbf249b2928c58296dfb23af94b7083d8455b exited with err_no: 0
```

Use the container id to inspect the runner for more details.

```
> devenv$ ./logs.sh 71c1beca7261e2c1b706fa0e9eeb3823ad56977c4846c89b25a315a48a1bbda5
```

the script will abort the moment a container fails. The script will print the
logs from the first failed container. You will need to stop the nodes yourself
with `docker stop $(docker ps -qa)`. You can also run `test` again to down and
up again any dangling container and reexecute tests once you have fixed and
rebuilt the testbed image.

### Running integration tests.

Start at devenv, `pushd` integration. You will find scripts in the `bin` folder
in this directory. The ones you will be using are `build` and `test`. `Test` is
how you are expected to run the suite. Run `bin/build` and `popd,` back in
devenv, and run `integration/bin/test`.

### Adding grouping filters

In /devenv/integration/test, there is a filter array that determines how many
nodes will be spun and what tests will run in parallel inside the node.

```bash
filters=("package(romeo)" "test(deposit_parse)" "test(deposit_output)")
```

Use Nextest's DSL to group up your integration tests. Add new filters as you see
fit.

### Adding node readiness checks.

In `devenv/integration/docker/entrypoint`, you can add checks to wait until a
node is ready to take tests.

In this snip, we wait until the stacks api is responsive and the burchain block
height is 205.

```bash
STACKS=$PROJECT_NAME-stacks-1
API_URL=http://$STACKS:20443/v2/info

# it makes sure the node is ready before proceeding
# stacks node get info
echo "Waiting on Stacks API"
while ! curl -s $API_URL >/dev/null; do
    sleep 1
done

DEV_READY_HEIGHT=205

# bitcoind get info
echo "Waiting on burn block height $DEV_READY_HEIGHT"
while [ "$(curl -s $API_URL | jq '.burn_block_height')" -lt $DEV_READY_HEIGHT ]; do
    sleep 2
done
```

### Troubleshooting.

- If a fresh network fails to be created or you spot a line like the one below, you need
  to stop all containers and prune the network.

```
! Network test_deposit__default          Resource is still in use                                                          0.0s
```

```
> docker stop $(docker ps -q)
> docker network prune
```

- The order of magnitude for the tests should be in **minutes**. It took 4mins
  in my system last time I checked.
