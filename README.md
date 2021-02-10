# Create Broken Files
This app try to broke files to allow test apps which import them etc. like e.g. [image-rs](https://github.com/image-rs/image/) which can crash when parsing valid or unvalid data, which is a bug because app should show error instead.

## Usage
After compiling you will get in `target/debug` or `target/release` an executable.

Let's say that we have one folder with two files:
```
/home/rafal/Pulpit/
  qaqa.txt
  roman.txt 
```
When we run `create_broken_files /home/rafal/Pulpit/qaqa.txt 2` there will be created 2 copies of provided file
```
/home/rafal/Pulpit/
  qaqa.txt
  qaqa1.txt
  qaqa2.txt
  roman.txt 
```
But when we run `create_broken_files /home/rafal/Pulpit/ 2` instead, then for each file inside this folder(only direct children without recursion) will be created additional copies:
```
/home/rafal/Pulpit/
  qaqa.txt
  qaqa1.txt
  qaqa2.txt
  roman.txt
  roman1.txt
  roman2.txt
```

## How it works

App choose a few random bytes in file which will replace.  
After replacing, file(usually broken) is saved to disk.  
There is ~10% chance that file will be saved with lower than normal number of bytes, e.g. 1MB instead 2MB.  
At the end values are restored, to be used by next iteration without copying big blocks of memory.
