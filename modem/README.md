Demodulation executes GNU radio flowgraphs under the hood. A Python virtual environment with conda is created with the required stuff.

Guide: https://wiki.gnuradio.org/index.php/CondaInstall

We shall create the environment directly in the `modem` directory instead. For this, when creating the environment instead run:

```sh
conda create -p <project_path>/modem/gnuradio
```

and active:

```sh
conda activate <project_path>/modem/gnuradio
```

Then install GNU Radio:

```sh
conda install gnuradio
```

TODO: is it necessary to install all gnu radio? Tbd.

I will provide some script or something to automate this in the future, or make it a crate that wraps gnu radio flowgraphs. Or maybe a Dockerfile for production use. For now, it works.

The above was tested on Windows. Should be easier on Linux tho.