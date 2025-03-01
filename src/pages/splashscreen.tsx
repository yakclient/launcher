import Image from "next/image";
import logo from "../public/icon.png"
import styles from "./splashscreen.module.sass"
import {useEffect, useState} from "react";
import {check} from "@tauri-apps/plugin-updater";
import {relaunch} from "@tauri-apps/plugin-process";
import {invoke} from "@tauri-apps/api/core";
import bg_png from "../../public/icons/login_bg.png";
import {Alert, Button} from "react-bootstrap";
import {useRouter} from "next/router";


const Splashscreen: React.FC = () => {
    let [rot, setRot] = useState(0);
    let [status, setStatus] = useState("Looking for updates...")
    const router = useRouter();

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
                }, 2000)
            } else {
                setStatus(`Everything is up to date!`)
                setTimeout(() => {
                    router.push("/authentication").then(() => {})
                }, 1000)
            }
        }).catch((e) => {
            console.log(e)
            setStatus(`Error attempting to update, starting launcher...`)
            setTimeout(() => {
                router.push("/authentication").then(() => {})
            }, 2000)
        });

        return () => clearInterval(interval);
    }, []);

    return <div id={styles.container}>
        <div id={styles.bg}>
            <Image
                src={bg_png}
                alt={"Background"}
                width={500}
                height={800}
                className={styles.title_image}
            />
        </div>
        <div id={styles.title} className={styles.centered}>
            <h1>YakClient</h1>
        </div>
        <h2 className={styles.centered}>Checking for updates</h2>
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
        <div className={styles.centered}>
            {status}
        </div>
    </div>

    // return <div id={styles.container}>
    //     <h1>YakClient</h1>
    //     <h2>Checking for updates</h2>
    //     <div id={styles.logo_container} style={{
    //         transform: "translate(-50%, 0) rotate(" + (-Math.pow(((rot % 40) - 20), 2) + 360) + "deg" + ")"
    //     }}>
    //         <Image width={100} height={100} src={
    //             logo
    //         } alt={"Logo"}/>
    //         <Image width={100} height={100} src={
    //             logo
    //         } alt={"Logo"}/>
    //     </div>
    //     <div>
    //         {status}
    //     </div>
    // </div>
}

export default Splashscreen