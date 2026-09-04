window.BENCHMARK_DATA = {
  "lastUpdate": 1788556444232,
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
      },
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
          "id": "17b0ec0897347ad00392d86cdba950e3cb7b4746",
          "message": "Merge pull request #9 from wiggels/fix-bench-cache\n\nci: wipe target/criterion before benching",
          "timestamp": "2026-09-04T15:53:13-05:00",
          "tree_id": "b979a26ac02f1970daba4d0bdf5a82fd35be3f87",
          "url": "https://github.com/wiggels/blockdev/commit/17b0ec0897347ad00392d86cdba950e3cb7b4746"
        },
        "date": 1788555375404,
        "tool": "cargo",
        "benches": [
          {
            "name": "get_devices/spawn_only",
            "value": 2720612,
            "range": "± 40040",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/parse_live_output",
            "value": 2183,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/full_request",
            "value": 2766495,
            "range": "± 70243",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/small_realistic",
            "value": 16054,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_human_size",
            "value": 189525,
            "range": "± 2800",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_byte_size",
            "value": 80023,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/size_string_heavy_1024",
            "value": 345552,
            "range": "± 1308",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_small",
            "value": 143,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "filters/non_system_filter_small",
            "value": 168,
            "range": "± 1",
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
            "value": 1307,
            "range": "± 22",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "6e75108f6388fcc38adaf37a93ac63ccb3188c80",
          "message": "Merge pull request #10 from wiggels/release-plz-2026-09-04T20-53-47Z\n\nchore: release v0.4.2",
          "timestamp": "2026-09-04T15:58:58-05:00",
          "tree_id": "d945e4d09f39b6566a983f65b39ddb2493be2b62",
          "url": "https://github.com/wiggels/blockdev/commit/6e75108f6388fcc38adaf37a93ac63ccb3188c80"
        },
        "date": 1788555687826,
        "tool": "cargo",
        "benches": [
          {
            "name": "get_devices/spawn_only",
            "value": 1335665,
            "range": "± 16253",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/parse_live_output",
            "value": 1403,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/full_request",
            "value": 1340815,
            "range": "± 49817",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/small_realistic",
            "value": 9208,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_human_size",
            "value": 109695,
            "range": "± 2258",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_byte_size",
            "value": 43284,
            "range": "± 1538",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/size_string_heavy_1024",
            "value": 190087,
            "range": "± 1453",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_small",
            "value": 99,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "filters/non_system_filter_small",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_hit",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_256",
            "value": 663,
            "range": "± 19",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "f58435a55c3f3ad496a05d7d7022e914d07d489e",
          "message": "Merge pull request #4 from wiggels/sysfs-backend\n\nfeat!: walk /sys directly, drop lsblk and JSON parsing",
          "timestamp": "2026-09-04T16:12:02-05:00",
          "tree_id": "640037a38374d1052017934fa32500140b6f2f0c",
          "url": "https://github.com/wiggels/blockdev/commit/f58435a55c3f3ad496a05d7d7022e914d07d489e"
        },
        "date": 1788556443749,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 1155301,
            "range": "± 25853",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 18753807,
            "range": "± 572561",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 3393,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 104,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 3279,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 694112,
            "range": "± 3866",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2910734,
            "range": "± 26322",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}