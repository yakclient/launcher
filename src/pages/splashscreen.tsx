import Image from "next/image";
import logo from "../public/icon.png"
import styles from "./splashscreen.module.sass"
import {useEffect, useState} from "react";
import {check} from "@tauri-apps/plugin-updater";
import {relaunch} from "@tauri-apps/plugin-process";
import {invoke} from "@tauri-apps/api/core";


const Splashscreen: React.FC = () => {
    let [rot, setRot] = useState(0);
    let [status, setStatus] = useState("Looking for updates...")

    useEffect(() => {
        let interval = setInterval(() => {
            setRot((rot) => {
                return rot + 0.1
            })
        }, 10)

        check().then(async (update) => {
            if (update) {
                setStatus(`Found update ${update.version} from ${update.date}`)
                let downloaded = 0;
                let contentLength = 0;
                // alternatively we could also call update.download() and update.install() separately
                await update.downloadAndInstall((event) => {
                    switch (event.event) {
                        case 'Started':
                            if (event.data.contentLength)
                                contentLength = event.data.contentLength;
                            setStatus(`Starting download`)
                            break;
                        case 'Progress':
                            downloaded += event.data.chunkLength;
                            setStatus(`${downloaded / contentLength * 100}% done`)
                            break;
                        case 'Finished':
                            console.log('download finished');
                            setStatus("Download finished, relaunching")
                            break;
                    }
                });

                setTimeout(() => {
                    relaunch();
                }, 1000)
            } else {
                setStatus(`Everything is up to date!`)
                setTimeout(() => {
                    invoke("leave_splashscreen").then(() => {
                        // Nothing
                    })
                }, 1000)
            }
        }).catch(() => {
            setStatus(`Error attempting to update, starting launcher...`)
            setTimeout(() => {
                invoke("leave_splashscreen").then(() => {
                    // Nothing
                })
            }, 3000)
        });

        return () => clearInterval(interval);
    }, []);

    return <div id={styles.container}>
        <h1>YakClient</h1>
        <h2>Checking for updates</h2>
        <div id={styles.logo_container} style={{
            transform: "translate(-50%, 0) rotate(" + (-Math.pow(((rot % 40) - 20), 2) + 360) + "deg" + ")"
        }}>
            <Image width={100} height={100} src={
                logo
            } alt={"Logo"}/>
            <Image width={100} height={100} src={
                logo
            } alt={"Logo"}/>
        </div>
        <div>
            {status}
        </div>
    </div>
}

export default Splashscreen