# The source tape file for creating the CLI intro GIF

# Setup env
Hide

Set TypingSpeed 50ms
Set Theme "GruvboxDark"
Set Shell fish

# boa_cli needs to be installed for the tape to run.
Require boa

Show

Type "boa"

Sleep 150ms

Enter

Sleep 500ms

Type "2 + '2'"

Sleep 150ms

Enter

Sleep 500ms

Type "const sayHello = () => { return 'Hello World!' };"

Sleep 150ms

Enter

Sleep 500ms

Type "sayHello();"

Sleep 150ms

Enter

Sleep 2s