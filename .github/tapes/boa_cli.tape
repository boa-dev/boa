Output cli/assets/boa_cli.gif

# Setup env
Hide

Set TypingSpeed 50ms
Set Theme "GruvboxDark"
Set Shell fish

# boa_cli needs to be installed for the tape to run.
Require boa

Show

Type "boa"

Sleep 100ms

Enter

Type "2 + '2'"

Enter

Sleep 100ms

Type "const sayHello = () => { return 'Hello World!' };"

Enter

Sleep 100ms

Type "sayHello();"

Enter

Sleep 2s
