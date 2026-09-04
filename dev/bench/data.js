window.BENCHMARK_DATA = {
  "lastUpdate": 1788554086184,
  "repoUrl": "https://github.com/wiggels/blockdev",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e9e7e4032bd9176e4bf0f5092a377c0e7d12d601",
          "message": "Merge pull request #2 from wiggels/ci-bench-infra\n\nci: benchmark history, perf budgets, and full workflow suite",
          "timestamp": "2026-09-04T15:31:51-05:00",
          "tree_id": "c03bca07d46f9e55b3664dabb6c56d2c6b3370e9",
          "url": "https://github.com/wiggels/blockdev/commit/e9e7e4032bd9176e4bf0f5092a377c0e7d12d601"
        },
        "date": 1788554085262,
        "tool": "cargo",
        "benches": [
          {
            "name": "get_devices/spawn_only",
            "value": 2683920,
            "range": "± 13743",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/parse_live_output",
            "value": 2387,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/full_request",
            "value": 2699489,
            "range": "± 12785",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/small_realistic",
            "value": 18124,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_human_size",
            "value": 219584,
            "range": "± 953",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_byte_size",
            "value": 87580,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/size_string_heavy_1024",
            "value": 436834,
            "range": "± 5660",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_small",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/non_system_filter_small",
            "value": 174,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_hit",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_256",
            "value": 1305,
            "range": "± 6",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}