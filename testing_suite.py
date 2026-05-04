#!/usr/bin/env python3
import os, subprocess, re
from datetime import datetime
from pathlib import Path

BIN = "./target/release/tms_loadtest"
ENDPOINTS = ["getversion", "getclient", "getkey", "createkey"]

def parse_req_fails(log_text: str):
    # Handle both formats:
    #   GET | 100 | 0 (0%)
    #   GET v1/tms/version | 100 | 0 (0%)
    m = re.search(
        r"^\s*(GET|POST|PUT|PATCH|DELETE)(?:\s+.*?)?\s*\|\s*([\d,]+)\s*\|\s*([\d,]+)\s*\(",
        log_text,
        re.MULTILINE,
    )
    if not m:
        return None, None
    reqs = int(m.group(2).replace(",", ""))
    fails = int(m.group(3).replace(",", ""))
    return reqs, fails

def run_once(run_root: Path, test_name: str, endpoint: str, users: int, iterations: int, throttle_requests=None):
    host = os.environ["TMS_URL"]

    outdir = run_root / test_name
    outdir.mkdir(parents=True, exist_ok=True)

    cmd = [
        BIN,
        "--users", str(users),
        "--iterations", str(iterations),
        "--scenarios", endpoint,
        "--host", host,
    ]
    if throttle_requests is not None:
        cmd += ["--throttle-requests", str(throttle_requests)]

    suffix = f"{endpoint}_u{users}_i{iterations}"
    if throttle_requests is not None:
        suffix += f"_tr{throttle_requests}"
    logfile = outdir / f"{suffix}.log"

    print("\n" + " ".join(cmd))
    p = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

    log_text = p.stdout
    logfile.write_text(log_text)

    reqs, fails = parse_req_fails(log_text)

    if reqs is None:
        if p.returncode != 0:
            reqs, fails = 1, 1  
        else:
            reqs, fails = 0, 0   

    fail_pct = 0.0 if reqs == 0 else (fails / reqs) * 100.0
    passed = (fails == 0 and p.returncode == 0)

    return {
        "test": test_name,
        "endpoint": endpoint,
        "users": users,
        "iterations": iterations,
        "throttle_requests": throttle_requests,
        "exit_code": p.returncode,
        "reqs": reqs,
        "fails": fails,
        "fail_pct": fail_pct,
        "passed": passed,
        "logfile": str(logfile),
    }

def smoke(run_root: Path, endpoints):
    results = []
    for ep in endpoints:
        results.append(run_once(run_root, "smoke", ep, users=1, iterations=100))
    return results

def ladder(run_root: Path, endpoints):
    results = []
    users_list = [1, 5, 10, 25, 50, 100]
    iterations = 10_00#0
    for ep in endpoints:
        for u in users_list:
            results.append(run_once(run_root, "ladder", ep, users=u, iterations=iterations))
    return results

def soak(run_root: Path, endpoints):
    results = []
    users = 25
    iterations = 200_0#00
    throttle_requests = 1000#200
    for ep in endpoints:
        results.append(
            run_once(
                run_root,
                "soak",
                ep,
                users=users,
                iterations=iterations,
                throttle_requests=throttle_requests,
            )
        )
    return results

def ask_yn(prompt: str) -> bool:
    return input(f"{prompt} [y/N]: ").strip().lower() in ("y", "yes")

def write_report(run_root: Path, results):
    report_path = run_root / "report.txt"

    lines = []
    lines.append(f"Run folder: {run_root}")
    lines.append(f"Total runs: {len(results)}")
    lines.append("")

    for r in results:
        status = "PASS" if r["fails"] == 0 else f"Fail-{r['fail_pct']:.2f}%"
        tr = "" if r["throttle_requests"] is None else f", throttle={r['throttle_requests']}"
        lines.append(
            f"[{status}] {r['test']}/{r['endpoint']} users={r['users']}, iters={r['iterations']}{tr} "
            f"reqs={r['reqs']} fails={r['fails']}  log={r['logfile']}"
        )

    lines.append("")
    overall_pass = all(r["fails"] == 0 for r in results) if results else True
    lines.append(f"OVERALL: {'PASS' if overall_pass else 'FAIL'}")

    report_path.write_text("\n".join(lines) + "\n")
    print(f"\nWrote report: {report_path}")

def main():
    ts = datetime.now().strftime("%Y%m%d%H%M")
    run_root = Path("loadtest_results") / ts
    run_root.mkdir(parents=True, exist_ok=True)

    selected_tests = []
    if ask_yn("Run smoke?"):
        selected_tests.append("smoke")
    if ask_yn("Run ladder?"):
        selected_tests.append("ladder")
    if ask_yn("Run soak?"):
        selected_tests.append("soak")

    selected_endpoints = []
    if ask_yn("Run getversion?"):
        selected_endpoints.append("getversion")
    if ask_yn("Run getclient?"):
        selected_endpoints.append("getclient")
    if ask_yn("Run getkey?"):
        selected_endpoints.append("getkey")
    if ask_yn("Run createkey?"):
        selected_endpoints.append("createkey")

    results = []
    for t in selected_tests:
        if t == "smoke":
            results += smoke(run_root, selected_endpoints)
        elif t == "ladder":
            results += ladder(run_root, selected_endpoints)
        elif t == "soak":
            results += soak(run_root, selected_endpoints)

    write_report(run_root, results)

if __name__ == "__main__":
    main()
