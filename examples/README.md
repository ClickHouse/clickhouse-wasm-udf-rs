# Examples

This directory contains example WebAssembly UDFs for ClickHouse, demonstrating how to implement and use custom functions written in Rust.

## Prerequisites

- Rust with the `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- A running ClickHouse server with WASM UDFs enabled

## Build

```sh
cargo build --release --target wasm32-unknown-unknown
```

This produces the `example.wasm` binary in `target/wasm32-unknown-unknown/release/`

```sh
file ../target/wasm32-unknown-unknown/release/example.wasm
# ../target/wasm32-unknown-unknown/release/example.wasm: WebAssembly (wasm) binary module version 0x1 (MVP)
```

## Load into ClickHouse

Upload the compiled module, then register each function.

```sh
# 1. Upload the .wasm binary
cat ../target/wasm32-unknown-unknown/release/example.wasm \
  | clickhouse client --query \
    "INSERT INTO system.webassembly_modules (name, code)
     SELECT 'example', code FROM input('code String') FORMAT RawBLOB"

# 2. Register the functions
clickhouse client --multiquery <<'SQL'
CREATE OR REPLACE FUNCTION greet
    LANGUAGE WASM ABI BUFFERED_V1
    FROM 'example'
    ARGUMENTS (name String)
    RETURNS String;

CREATE OR REPLACE FUNCTION parse_pem
    LANGUAGE WASM ABI BUFFERED_V1
    FROM 'example'
    ARGUMENTS (s String)
    RETURNS Map(String, String);
SQL
```

## Try it

```sql
SELECT greet('ClickHouse');
-- Hello, ClickHouse!

SELECT greet('')
-- Code: 770. DB::Exception: Received from localhost:9000. DB::Exception: WebAssembly UDF terminated with error: Name cannot be empty: while executing 'FUNCTION greet(''_String :: 1) -> greet(''_String) String : 2'. (WASM_ERROR)
```


```sql
SELECT
    m['subject'] as subject,
    toDateTime(toUInt64OrZero(m['not_before']), 'Europe/London') as not_before,
    toDateTime(toUInt64OrZero(m['not_after']), 'Europe/London') as not_after, 
    m['issuer'] as issuer
FROM (
    -- Command to get certificate: "openssl s_client -connect clickhouse.com:443 </dev/null | openssl x509 -outform PEM"
    SELECT parse_pem(concat(
        '-----BEGIN CERTIFICATE-----', '\n',
        'MIIDsDCCA1agAwIBAgIRAKs4Xe4kUlfQDWpx0zRcEqwwCgYIKoZIzj0EAwIwOzELMAkGA1UEBhMCVVMxHjAcBgNVBAoTFUdvb2dsZSBUcnVzdCBTZXJ2aWNlczEMMAoG',
        'A1UEAxMDV0UxMB4XDTI0MDgyOTEwMTIxM1oXDTI0MTEyNzEwNDUzMlowGTEXMBUGA1UEAxMOY2xpY2tob3VzZS5jb20wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQR',
        'O7WGBt5RbM4SpNB/C5Xx/j2R1mPGgyeYlgoJ4DXHRV2ucoYN2c6fv//PyxaoZuN2qH66hISD0SlzNq6PhzEbo4ICWzCCAlcwDgYDVR0PAQH/BAQDAgeAMBMGA1UdJQQM',
        'MAoGCCsGAQUFBwMBMAwGA1UdEwEB/wQCMAAwHQYDVR0OBBYEFKo3LRQUsE3ZTfHrjEQy8miAswyOMB8GA1UdIwQYMBaAFJB3kjVnxP+ozKnme9mAeXvMk/k4MF4GCCsG',
        'AQUFBwEBBFIwUDAnBggrBgEFBQcwAYYbaHR0cDovL28ucGtpLmdvb2cvcy93ZTEvcXpnMCUGCCsGAQUFBzAChhlodHRwOi8vaS5wa2kuZ29vZy93ZTEuY3J0MC4GA1Ud',
        'EQQnMCWCDmNsaWNraG91c2UuY29tghNkYXRhLmNsaWNraG91c2UuY29tMBMGA1UdIAQMMAowCAYGZ4EMAQIBMDYGA1UdHwQvMC0wK6ApoCeGJWh0dHA6Ly9jLnBraS5n',
        'b29nL3dlMS9sSXR3RTdGSE9PTS5jcmwwggEDBgorBgEEAdZ5AgQCBIH0BIHxAO8AdQDuzdBk1dsazsVct520zROiModGfLzs3sNRSFlGcR+1mwAAAZGd1gQsAAAEAwBG',
        'MEQCIGhzkQ7fKdn781DmIfVvW36UAVSpzylnI1spVrXF4QetAiBPQ0YX7ok04aAIjyxhJOS3A2vANTk3myDLQgXw+xFWxwB2ANq2v2s/tbYin5vCu1xr6HCRcWy7UYSF',
        'NL2kPTBI1/urAAABkZ3WBGMAAAQDAEcwRQIhALqJt9Y33HXkwMxa6vBjPtb1hFt8bnih8I4MVQwssw63AiAkUsBgPRbvdVdUsYL6TG2OLDbqsYDWam01ktFpQQoaSjAK',
        'BggqhkjOPQQDAgNIADBFAiBH4WNNpPaoCt+K8zcafamSvtMKmg7bJg+2T/Cs4+zC9wIhAK+yb2CTZOKxsMUkP6s3QW7tKBLC0BWMIgmO4iGRssZ2', '\n',
        '-----END CERTIFICATE-----')) as m
);

--    ┌─subject───────────┬──────────not_before─┬───────────not_after─┬─issuer────────────────────────────────┐
-- 1. │ CN=clickhouse.com │ 2024-08-29 11:12:13 │ 2024-11-27 10:45:32 │ C=US, O=Google Trust Services, CN=WE1 │
--    └───────────────────┴─────────────────────┴─────────────────────┴───────────────────────────────────────┘
```
