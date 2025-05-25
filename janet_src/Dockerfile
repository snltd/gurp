FROM leafgarland/janet-sdk
COPY janet_src /janet_src
RUN jpm install judge
ENTRYPOINT ["judge", "/janet_src"]
