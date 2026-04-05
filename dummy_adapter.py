import sys
import json
import time

def print_result(status, fingerprint=None, error_reason=None):
    result = {
        "status": status,
        "version": "0.1.1-mock",
        "fingerprint": fingerprint,
        "error_reason": error_reason
    }
    print(json.dumps(result))
    sys.exit(0)

def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "normal"
    
    # Read the raw bytes just to prove the pipe works
    input_bytes = sys.stdin.buffer.read()

    try:
        if mode == "normal":
            # Simulate a successful parse and fingerprint generation
            print_result("accept", fingerprint=f"mock_fingerprint_for_{len(input_bytes)}_bytes")
            
        elif mode == "hang":
            # Simulate a parser caught in a malformed recursive loop
            time.sleep(5) 
            print_result("accept", fingerprint="you_should_never_see_this")
            
        elif mode == "crash":
            # Simulate a memory out-of-bounds or deep syntax crash
            raise ValueError("Unexpected byte 0xFF at offset 42")

    except Exception as e:
        print_result("crash", error_reason=f"panic: {str(e)}")

if __name__ == "__main__":
    main()