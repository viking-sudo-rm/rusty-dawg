# Below are different runs that I restarted manually.

# Restart runs that failed due to wrong key.
SPLIT=06 HOST=prior-cirrascale-64.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=08 HOST=prior-cirrascale-65.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=16 HOST=prior-cirrascale-66.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=22 HOST=allennlp-cirrascale-68.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=29 HOST=climate-cirrascale-72.reviz.ai2.in beaker experiment create beaker/pile.yaml

# These runs never started, so I restarted them.
SPLIT=24 HOST=s2-elanding-22.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=25 HOST=s2-elanding-24.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=26 HOST=allennlp-elanding-30.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=27 HOST=mosaic-elanding-33.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=28 HOST=mosaic-elanding-34.reviz.ai2.in beaker experiment create beaker/pile.yaml

# These runs eventually exited with 137 error codes.
SPLIT=20 HOST=general-cirrascale-73.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=23 HOST=prior-cirrascale-74.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=02 HOST=prior-elanding-52.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=03 HOST=prior-elanding-54.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=04 HOST=prior-elanding-55.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=05 HOST=prior-elanding-56.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=12 HOST=aristo-elanding-57.reviz.ai2.in beaker experiment create beaker/pile.yaml
SPLIT=15 HOST=aristo-elanding-58.reviz.ai2.in beaker experiment create beaker/pile.yaml

# Cancelled and moved to different machine because it was competing with @minyoungh's experiment for memory.
SPLIT=19 HOST=prior-elanding-59.reviz.ai2.in beaker experiment create beaker/pile.yaml
