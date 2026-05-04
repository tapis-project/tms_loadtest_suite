# tms_loadtesting_suite

## Needed Exports

```bash
export TMS_PARSE_RESPONSE=true
export X_TMS_CLIENT_ID=testclient1
export X_TMS_CLIENT_SECRET=secret1
export X_TMS_TENANT=test
export TMS_PUBKEY_FINGERPRINT="SHA256:Y1nwRJ82LwfEwr2cxe48pB5fzqBK3DoKK4DuH584pDw"
export TMS_PUBKEY_HOST="testhost1"
export TMS_PUBKEY_KEYTYPE="ed25519"
export TMS_PUBKEY_USER="testhostaccount1"
export TMS_PUBKEY_USERID="1"
```

- If running against dev server:
```bash
export TMS_URL=https://tms-server-dev.tacc.utexas.edu:3000
```

- If running on a local instance of `tms_server`
```bash
export TMS_URL="http://127.0.0.1:3000"
```
- or whatever port you have it running on
- Note: `getkey` test does not work when running locally

## Running the script
```bash
python3 testing_suite.py
```
- You will be prompted with 7 yes or no questions that determine which tests you want to run
```bash
Run smoke? [y/N]: 
Run ladder? [y/N]: 
Run soak? [y/N]: 
Run getversion? [y/N]: 
Run getclient? [y/N]: 
Run getkey? [y/N]: 
Run createkey? [y/N]:
```

 ### What Each Question Means
 - Smoke: Quick test to see if it is working
 - Ladder: Increases load across multiple tests, can use this too see if it starts to fail at some test
 - Soak: Long throttled test to emulate real consistent usage
 - `getversion`, `getclient`, `getkey`, `creatkey`: The four different endpoints we can test

### Example
```bash
Run smoke? [y/N]: y
Run ladder? [y/N]: y
Run soak? [y/N]: n
Run getversion? [y/N]: y
Run getclient? [y/N]: y
Run getkey? [y/N]: n
Run createkey? [y/N]:n
```
- This will run the smoke test and ladder test for both the `getversion` and `getclient` endpoints

## Example Workflow
```bash
tjo656@tms-loadtest:~/tms_loadtesting_suite$ export TMS_PARSE_RESPONSE=true
export X_TMS_CLIENT_ID=testclient1
export X_TMS_CLIENT_SECRET=secret1
export X_TMS_TENANT=test
export TMS_PUBKEY_FINGERPRINT="SHA256:Y1nwRJ82LwfEwr2cxe48pB5fzqBK3DoKK4DuH584pDw"
export TMS_PUBKEY_HOST="testhost1"
export TMS_PUBKEY_KEYTYPE="ed25519"
export TMS_PUBKEY_USER="testhostaccount1"
export TMS_PUBKEY_USERID="1"
tjo656@tms-loadtest:~/tms_loadtesting_suite$ export TMS_URL=https://tms-server-dev.tacc.utexas.edu:3000
tjo656@tms-loadtest:~/tms_loadtesting_suite$ python3 testing_suite.py 
Run smoke? [y/N]: y
Run ladder? [y/N]: n
Run soak? [y/N]: n
Run getversion? [y/N]: y
Run getclient? [y/N]: y
Run getkey? [y/N]: y
Run createkey? [y/N]: y

./target/release/tms_loadtest --users 1 --iterations 100 --scenarios getversion --host https://tms-server-dev.tacc.utexas.edu:3000

./target/release/tms_loadtest --users 1 --iterations 100 --scenarios getclient --host https://tms-server-dev.tacc.utexas.edu:3000

./target/release/tms_loadtest --users 1 --iterations 100 --scenarios getkey --host https://tms-server-dev.tacc.utexas.edu:3000

./target/release/tms_loadtest --users 1 --iterations 100 --scenarios createkey --host https://tms-server-dev.tacc.utexas.edu:3000

Wrote report: loadtest_results/202603241418/report.txt
tjo656@tms-loadtest:~/tms_loadtesting_suite$ cd loadtest_results/202603241418/
tjo656@tms-loadtest:~/tms_loadtesting_suite/loadtest_results/202603241418$ cat report.txt 
Run folder: loadtest_results/202603241418
Total runs: 4

[PASS] smoke/getversion users=1, iters=100 reqs=100 fails=0  log=loadtest_results/202603241418/smoke/getversion_u1_i100.log
[PASS] smoke/getclient users=1, iters=100 reqs=100 fails=0  log=loadtest_results/202603241418/smoke/getclient_u1_i100.log
[PASS] smoke/getkey users=1, iters=100 reqs=100 fails=0  log=loadtest_results/202603241418/smoke/getkey_u1_i100.log
[PASS] smoke/createkey users=1, iters=100 reqs=100 fails=0  log=loadtest_results/202603241418/smoke/createkey_u1_i100.log

OVERALL: PASS
tjo656@tms-loadtest:~/tms_loadtesting_suite/loadtest_results/202603241418$ ls
report.txt  smoke
tjo656@tms-loadtest:~/tms_loadtesting_suite/loadtest_results/202603241418$ cd smoke/
tjo656@tms-loadtest:~/tms_loadtesting_suite/loadtest_results/202603241418/smoke$ ls
createkey_u1_i100.log  getclient_u1_i100.log  getkey_u1_i100.log  getversion_u1_i100.log
tjo656@tms-loadtest:~/tms_loadtesting_suite/loadtest_results/202603241418/smoke$ 
```
