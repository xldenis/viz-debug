echo '/dev/ttyACM0 9990 /dev/ttyACM1 9991 /dev/ttyACM2 9992 /dev/ttyACM3 9993 /dev/ttyACM4 9994' | xargs -P 10 -n2 cargo run server
