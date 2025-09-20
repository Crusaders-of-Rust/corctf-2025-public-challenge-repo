import subprocess

pattern = "ananas"
source = pattern * 7

for i in range(len(source)):
    attempt = "corctf{" + source[0:i] + "}\n"
    res = subprocess.run("./tagme", input=attempt, text=True, capture_output=True)
    message = str(res.stdout).split('\n')[-3]
    success = res.returncode == 0
    if success:
        print(f"Success! {attempt = } : {message}")
    else:
        print(f"Failure. {attempt = } : {message}")