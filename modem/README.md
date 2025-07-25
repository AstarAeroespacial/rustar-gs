Demodulation executes GNU Radio flowgraphs under the hood. The host system must have a working GNU Radio installation, either on the global scope or in a virtual environment.

I will now provide instructions for setting up GNU Radio in a virtual environment, both on Linux and Windows.

# On Windows

A Python virtual environment with conda is created with the required stuff.

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

TODO: is it necessary to install all of GNU Radio? Maybe we can avoid having to install all the GUI stuff.

# On Linux
TODO: write instructions for installing GNU Radio in a virtual environment on Linux.

NOTE: I will provide some script or something to automate this in the future, or make it a crate that wraps GNU Radio flowgraphs. Or maybe a Dockerfile for production use. For now, it works.


# Adding flowgraphs

For now, the demodulator is hardcoded to send the IQ samples to a ZMQ socket at address `tcp://127.0.0.1:5556`, and read the resulting bits from `tcp://127.0.0.1:5557`. This is temporary, the idea is to create a better abstraction over the execution of flowgraphs.