window.BENCHMARK_DATA = {
  "lastUpdate": 1780777890122,
  "repoUrl": "https://github.com/suonengine/suon",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "committer": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "distinct": true,
          "id": "108e2aee63b42060b1910ece896227602270a468",
          "message": "chore: initial commit",
          "timestamp": "2026-06-06T17:28:54-03:00",
          "tree_id": "cfbf22c1d5a59cb90d80b233dd659039e9774f39",
          "url": "https://github.com/suonengine/suon/commit/108e2aee63b42060b1910ece896227602270a468"
        },
        "date": 1780777889576,
        "tool": "cargo",
        "benches": [
          {
            "name": "xtea_expand_key",
            "value": 26,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_encrypt_8_bytes",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt_8_bytes",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_encrypt_1024_bytes",
            "value": 4962,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt_1024_bytes",
            "value": 5041,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_encrypt_1_mebibyte",
            "value": 5101175,
            "range": "± 11170",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt_1_mebibyte",
            "value": 5211836,
            "range": "± 15056",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_roundtrip_1_mebibyte",
            "value": 10298308,
            "range": "± 11357",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}