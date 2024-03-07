# forcedmode
rust to show forcing of mode changes in object transformations

This is a mockup project to demonstrate how we can enforce proper mode
transitions at compile time where it's not possible to write code that
might attempt to manipulate hardware in a way that would be inconsistant
with proper state transitions.

This goes beyond runtime checks that a mode transition is correct, by using
consuming method calls that _optionally_ return an implementation of a
new trait.

There is a slight increase in proper handling of the values so that it
is not inadvertantly lost, but I'd argue that this is a necessary complexity
that is required of any implementation.  An implementation that isn't enforcing
it at compile time cannot detect lax resource management, and therefore the
implementation of correct behaviour cannot be guarenteed without language
features of ownership transfer.