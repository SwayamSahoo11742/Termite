
import sys
import subprocess
import os


obj = sys.argv[1]
# Get the current working directory
current_path = os.getcwd()

# Run the "t3d" command in WSL from the current directory, ensuring a proper bash environment
subprocess.run(["t3d", obj], cwd=current_path)
