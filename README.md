A telegram bot canister
=======================

This Internet Computer canister speaks Telegram! It is possible to host a simple
Telegram bot on the Internet Computer, without any external infrastructure needed.

[HTTP Gateway]: https://github.com/nomeata/ic-http-lambda


Historical note
---------------

When this bot was created, in December 2020, the official HTTP Gateway of the
Internet Computer did not yet exist, and I was using a [simple HTTP
Gateway](https://github.com/nomeata/ic-http-lambda) that I hosted on AWS myself
to run this bot. This served as a proof of concept and inspiration for the
official HTTP Gateway feature of the Internet Computer.  That only gained the
feature to perform state-changing request in March 2021.  Now the Telegram bot
is completely hosted on the Internet Computer.
