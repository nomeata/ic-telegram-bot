#!/usr/bin/env bash

canister_id="$(jq -r .telegram.ic < canister_ids.json)"

if [ -z "$canister_id" ]; then
  echo "Could not read canister id for canister \"telegram\" from ./canister_ids"
  exit 1
fi

token="$(cat token)"
if [ -z "$token" ]; then
  echo "Could not read file ./token"
  exit 1
fi

curl "https://api.telegram.org/bot$token/setWebhook?url=https://$canister_id.ic.nomeata.de/webhook/$token"
