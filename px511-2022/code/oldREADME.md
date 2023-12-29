# Password Manager

## INSTALLATION  

The script hasn't been fully tested yet.
```
cd Server
sudo ./INIT.sh

Enter "1" in the terminal.
```

## STARTING/STOPPING THE SERVER
The server runs of the port 40443.
```
cd Server
./START.sh
./STOP.sh
```

## WEB TESTS

```
cd Server
cargo test [category]
```

>\[category\] : \"account\"   
>Example : cargo test account


## DOCUMENTATION

```
cd Server or cd Client
cargo doc --open
```