import numpy as np

from utils.dawg_results import DawgResults
from utils.retokenizer import Retokenizer

res = DawgResults.load("/home/willm/dawg-results/models-10")
res = res.trim(n_tokens=10000)

