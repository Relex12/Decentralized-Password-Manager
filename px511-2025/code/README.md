# 3p-nike

## Getting started

To clone this project on your repository,
Do the following command: 
```
git clone https://gricad-gitlab.univ-grenoble-alpes.fr/senegasm/3p-nike.git
``` 

## Library

To execute the code of implementation 3p-nike, you have to install bplib.
To that, create a python environment and then install bplib :

```
python3 -m venv venv
pip install bplib
source venv/bin/activate
```

## Run the implementation

There are two folders.
- 3Party-Diffie–Hellman-2Rounds
- 3Party-NIKE-library

The first is used to run the implementation in 2 rounds with Diffie Helman, like presented in the intermediate Rapport.
The second, the real implementation, uses the pairing and the theory's Joux.

To run the second, after you've sourced the venv, launch 

```
python3 ./3Party-NIKE-library/main.py
```

In console, you will see the result of the implementation.

## Other Branchs

In matthieu Branch, there are multiple tests of implementation of the 3p-nike without library using pairing.
