from kokoro import KPipeline
import soundfile as sf
import torch
pipeline = KPipeline(lang_code='a')
text = '''
Why does TUCaN load so slowly? Why can't I share URLs with other students? Why is it so bad on mobile? Why is the registration menu so slow?
If you have been asking yourself the same, we have a solution for you. We introduce TUCaN Plus the best extension to make TUCaN can again. It will remove questionable half a second waits in the code of TUCaN and skip unecessary navigations. Also it works nicely on mobile with a completely new user interface. It also caches pages you already viewed before so TUCaN is not always so slow. Unfortunately we can't fix it being slow the first time.
But let's look at the features in detail:
How do I install this cool extension?
Go to https://tucan-plus.github.io/tucan-plus/. Then, click on download extension for Firefox. Now, confirm the installation prompts. How do I configure TUCaN Plus? Click on the extension icon in the top right and select TUCaN Plus. Now click on Go to options.
'''
generator = pipeline(text, voice='af_heart')
for i, (gs, ps, audio) in enumerate(generator):
    print(i, gs, ps)
    sf.write(f'{i}.wav', audio, 24000)