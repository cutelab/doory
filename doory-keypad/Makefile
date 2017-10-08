all: ../target/thumbv7m-none-eabi/release/doory-keypad

.PHONY: program clean

../target/thumbv7m-none-eabi/release/doory-keypad: src/main.rs
	xargo build --release

program: ../target/thumbv7m-none-eabi/release/doory-keypad
	arm-none-eabi-gdb ../target/thumbv7m-none-eabi/release/doory-keypad --batch-silent
	echo 'press reset on bluepill'

clean:
	xargo clean
