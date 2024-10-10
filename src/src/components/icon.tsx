//===================================================================
// Import References 
//===================================================================
import React from "react";

//===================================================================
// Constant Variables Definitions
//===================================================================
const iconsPath = `${process.env.PUBLIC_URL}/icons`

//===================================================================
// Export Type Definitions
//===================================================================

//===================================================================
// Local Type Definitions
//===================================================================
type Props = {
    name: IconName,
    width?: number,
    height?: number
}

//===================================================================
// Class Definitions
//===================================================================
export enum IconName {
    Add,
    Cog,
    DownloadFile,
    Edit,
    FloppyDisk,
    NewFolder,
    RecycleBin,
    UploadFile,
    Variation,
    EyeOpen,
    BorderedWindow,
    Close,
    Fullscreen,
    Minimize
}

//===================================================================
// Function Definitions
//===================================================================

//===================================================================
// Component Definition
//===================================================================
const iconMap = {
    [IconName.Add]: <svg width="9" height="9" viewBox="0 0 9 9" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M4.71429 1V8.42857" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M1 4.69141H8.42857" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.BorderedWindow]: <svg width="19" height="19" viewBox="0 0 19 19" fill="none"
                                    xmlns="http://www.w3.org/2000/svg">
        <g filter="url(#filter0_d_9_27)">
            <path
                d="M7.57692 3.57692V1.26923C7.57692 0.850256 7.92718 0.5 8.34615 0.5H13.7308C14.1497 0.5 14.5 0.850256 14.5 1.26923V6.65385C14.5 7.07282 14.1497 7.42308 13.7308 7.42308H11.4231"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path
                d="M5.26925 10.5C4.8444 10.5 4.50002 10.1556 4.50002 9.73078V4.34617C4.50002 3.92719 4.85025 3.57693 5.26925 3.57693H10.6539C11.0728 3.57693 11.4231 3.92719 11.4231 4.34617V9.73078C11.4231 10.1498 11.0728 10.5 10.6539 10.5H5.26925Z"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
        <defs>
            <filter id="filter0_d_9_27" x="0" y="0" width="19" height="19" filterUnits="userSpaceOnUse"
                    color-interpolation-filters="sRGB">
                <feFlood flood-opacity="0" result="BackgroundImageFix"/>
                <feColorMatrix in="SourceAlpha" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
                               result="hardAlpha"/>
                <feOffset dy="4"/>
                <feGaussianBlur stdDeviation="2"/>
                <feComposite in2="hardAlpha" operator="out"/>
                <feColorMatrix type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.25 0"/>
                <feBlend mode="normal" in2="BackgroundImageFix" result="effect1_dropShadow_9_27"/>
                <feBlend mode="normal" in="SourceGraphic" in2="effect1_dropShadow_9_27" result="shape"/>
            </filter>
        </defs>
    </svg>,
    [IconName.Close]: <svg width="11" height="11" viewBox="0 0 11 11" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M10.5 0.5L0.5 10.5" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M0.5 0.5L10.5 10.5" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.Cog]: <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
        <g clip-path="url(#clip0_9_42)">
            <path
                d="M5.23004 2.25L5.66004 1.14C5.73256 0.952064 5.86015 0.790411 6.02609 0.676212C6.19204 0.562014 6.3886 0.500595 6.59004 0.5H7.41004C7.61148 0.500595 7.80805 0.562014 7.97399 0.676212C8.13994 0.790411 8.26752 0.952064 8.34004 1.14L8.77004 2.25L10.23 3.09L11.41 2.91C11.6065 2.88333 11.8065 2.91567 11.9846 3.00292C12.1626 3.09017 12.3107 3.22838 12.41 3.4L12.81 4.1C12.9125 4.27435 12.9598 4.47568 12.9455 4.67742C12.9312 4.87916 12.8561 5.07183 12.73 5.23L12 6.16V7.84L12.75 8.77C12.8761 8.92817 12.9512 9.12084 12.9655 9.32258C12.9798 9.52432 12.9325 9.72565 12.83 9.9L12.43 10.6C12.3307 10.7716 12.1826 10.9098 12.0046 10.9971C11.8265 11.0843 11.6265 11.1167 11.43 11.09L10.25 10.91L8.79004 11.75L8.36004 12.86C8.28752 13.0479 8.15994 13.2096 7.99399 13.3238C7.82805 13.438 7.63148 13.4994 7.43004 13.5H6.59004C6.3886 13.4994 6.19204 13.438 6.02609 13.3238C5.86015 13.2096 5.73256 13.0479 5.66004 12.86L5.23004 11.75L3.77004 10.91L2.59004 11.09C2.39356 11.1167 2.19358 11.0843 2.01552 10.9971C1.83747 10.9098 1.68937 10.7716 1.59004 10.6L1.19004 9.9C1.08754 9.72565 1.04032 9.52432 1.0546 9.32258C1.06888 9.12084 1.144 8.92817 1.27004 8.77L2.00004 7.84V6.16L1.25004 5.23C1.124 5.07183 1.04888 4.87916 1.0346 4.67742C1.02032 4.47568 1.06754 4.27435 1.17004 4.1L1.57004 3.4C1.66937 3.22838 1.81747 3.09017 1.99552 3.00292C2.17358 2.91567 2.37356 2.88333 2.57004 2.91L3.75004 3.09L5.23004 2.25ZM5.00004 7C5.00004 7.39556 5.11734 7.78224 5.3371 8.11114C5.55687 8.44004 5.86922 8.69638 6.23467 8.84776C6.60013 8.99913 7.00226 9.03874 7.39022 8.96157C7.77818 8.8844 8.13455 8.69392 8.41426 8.41421C8.69396 8.13451 8.88444 7.77814 8.96161 7.39018C9.03878 7.00222 8.99918 6.60009 8.8478 6.23463C8.69643 5.86918 8.44008 5.55682 8.11118 5.33706C7.78228 5.1173 7.3956 5 7.00004 5C6.46961 5 5.9609 5.21071 5.58583 5.58579C5.21076 5.96086 5.00004 6.46957 5.00004 7Z"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
        <defs>
            <clipPath id="clip0_9_42">
                <rect width="14" height="14" fill="white"/>
            </clipPath>
        </defs>
    </svg>,
    [IconName.DownloadFile]: <svg width="14" height="16" viewBox="0 0 14 16" fill="none"
                                  xmlns="http://www.w3.org/2000/svg">
        <path
            d="M13 13.9231C13 14.2087 12.885 14.4826 12.6805 14.6846C12.4759 14.8865 12.1984 15 11.9091 15H2.09091C1.80158 15 1.52411 14.8865 1.31952 14.6846C1.11494 14.4826 1 14.2087 1 13.9231V2.07692C1 1.7913 1.11494 1.51739 1.31952 1.31542C1.52411 1.11346 1.80158 1 2.09091 1H9.18182L13 4.76923V13.9231Z"
            stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M9.18181 9.07693L6.99999 11.2308L4.81818 9.07693" stroke="currentColor" stroke-linecap="round"
              stroke-linejoin="round"/>
        <path d="M7 11.2308V5.30768" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.Edit]: <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
        <g clip-path="url(#clip0_78_221)">
            <path
                d="M5 12.24L0.5 13.5L1.76 9.00002L10 0.80002C10.0931 0.704774 10.2044 0.629095 10.3271 0.577428C10.4499 0.52576 10.5818 0.499146 10.715 0.499146C10.8482 0.499146 10.9801 0.52576 11.1029 0.577428C11.2256 0.629095 11.3369 0.704774 11.43 0.80002L13.2 2.58002C13.2937 2.67298 13.3681 2.78359 13.4189 2.90544C13.4697 3.0273 13.4958 3.15801 13.4958 3.29002C13.4958 3.42203 13.4697 3.55274 13.4189 3.6746C13.3681 3.79646 13.2937 3.90706 13.2 4.00002L5 12.24Z"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
        <defs>
            <clipPath id="clip0_78_221">
                <rect width="14" height="14" fill="white"/>
            </clipPath>
        </defs>
    </svg>,
    [IconName.EyeOpen]: <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path
            d="M13.23 6.2463C13.3958 6.45302 13.4876 6.72159 13.4876 7.00006C13.4876 7.27853 13.3958 7.5471 13.23 7.75382C12.18 9.02509 9.78997 11.5001 6.99997 11.5001C4.20997 11.5001 1.81997 9.02509 0.769968 7.75382C0.604128 7.5471 0.512329 7.27853 0.512329 7.00006C0.512329 6.72159 0.604128 6.45302 0.769968 6.2463C1.81997 4.97503 4.20997 2.5 6.99997 2.5C9.78997 2.5 12.18 4.97503 13.23 6.2463Z"
            stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M7 9C8.10457 9 9 8.10457 9 7C9 5.89543 8.10457 5 7 5C5.89543 5 5 5.89543 5 7C5 8.10457 5.89543 9 7 9Z"
              stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.FloppyDisk]: <svg width="16" height="16" stroke="currentColor" viewBox="0 0 16 16" fill="none"
                                xmlns="http://www.w3.org/2000/svg">
        <path
            d="M15 13.9231C15 14.2087 14.8865 14.4826 14.6846 14.6846C14.4826 14.8865 14.2087 15 13.9231 15H2.07692C1.7913 15 1.51739 14.8865 1.31542 14.6846C1.11346 14.4826 1 14.2087 1 13.9231V5.74923C1.00119 5.46671 1.11335 5.19597 1.31231 4.99538L4.99538 1.31231C5.19597 1.11335 5.46671 1.00119 5.74923 1H13.9231C14.2087 1 14.4826 1.11346 14.6846 1.31542C14.8865 1.51739 15 1.7913 15 2.07692V13.9231Z"
            stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        <path
            d="M12 15V10.5C12 10.3674 11.9473 10.2402 11.8536 10.1465C11.7598 10.0527 11.6326 10 11.5 10H5.5C5.36739 10 5.24021 10.0527 5.14645 10.1465C5.05268 10.2402 5 10.3674 5 10.5V15"
            stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        <path
            d="M12 2V5.5C12 5.63261 11.9473 5.75979 11.8536 5.85355C11.7598 5.94732 11.6326 6 11.5 6H7.5C7.36739 6 7.24021 5.94732 7.14645 5.85355C7.05268 5.75979 7 5.63261 7 5.5V2"
            stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.Fullscreen]: <svg width="14" height="14" viewBox="0 0 14 14" fill="none"
                                xmlns="http://www.w3.org/2000/svg">
        <g clip-path="url(#clip0_1222_47168)">
            <path
                d="M12.5 0.5H1.5C0.947715 0.5 0.5 0.947715 0.5 1.5V12.5C0.5 13.0523 0.947715 13.5 1.5 13.5H12.5C13.0523 13.5 13.5 13.0523 13.5 12.5V1.5C13.5 0.947715 13.0523 0.5 12.5 0.5Z"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
        <defs>
            <clipPath id="clip0_1222_47168">
                <rect width="14" height="14" fill="white"/>
            </clipPath>
        </defs>
    </svg>,
    [IconName.Minimize]: <svg width="11" height="2" viewBox="0 0 11 2" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M0.5 1H10.5" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.NewFolder]: <svg width="16" height="14" viewBox="0 0 16 14" fill="none"
                               xmlns="http://www.w3.org/2000/svg">
        <path
            d="M1 12.04V1.96038C1 1.70578 1.11346 1.46161 1.31542 1.28158C1.51739 1.10155 1.7913 1.00041 2.07692 1.00041H6.05077C6.29677 0.994019 6.53782 1.06293 6.73377 1.19568C6.9297 1.32842 7.06873 1.51699 7.12769 1.72999L7.46154 2.92035H13.9231C14.2087 2.92035 14.4826 3.02148 14.6846 3.20152C14.8865 3.38155 15 3.62572 15 3.88032V12.04C15 12.2947 14.8865 12.5388 14.6846 12.7188C14.4826 12.8989 14.2087 13 13.9231 13H2.07692C1.7913 13 1.51739 12.8989 1.31542 12.7188C1.11346 12.5388 1 12.2947 1 12.04Z"
            stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.RecycleBin]: <svg width="14" height="14" viewBox="0 0 14 14" fill="none"
                                xmlns="http://www.w3.org/2000/svg">
        <g clip-path="url(#clip0_10_72)">
            <path d="M1 3.5H13" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path
                d="M2.5 3.5H11.5V12.5C11.5 12.7652 11.3946 13.0196 11.2071 13.2071C11.0196 13.3946 10.7652 13.5 10.5 13.5H3.5C3.23478 13.5 2.98043 13.3946 2.79289 13.2071C2.60536 13.0196 2.5 12.7652 2.5 12.5V3.5Z"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path
                d="M4.5 3.5V3C4.5 2.33696 4.76339 1.70107 5.23223 1.23223C5.70107 0.763392 6.33696 0.5 7 0.5C7.66304 0.5 8.29893 0.763392 8.76777 1.23223C9.23661 1.70107 9.5 2.33696 9.5 3V3.5"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M5.5 6.50146V10.503" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M8.5 6.50146V10.503" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
        <defs>
            <clipPath id="clip0_10_72">
                <rect width="14" height="14" fill="white"/>
            </clipPath>
        </defs>
    </svg>,
    [IconName.UploadFile]: <svg width="14" height="16" viewBox="0 0 14 16" fill="none"
                                xmlns="http://www.w3.org/2000/svg">
        <path
            d="M13 13.9231C13 14.2087 12.885 14.4826 12.6805 14.6846C12.4759 14.8865 12.1984 15 11.9091 15H2.09091C1.80158 15 1.52411 14.8865 1.31952 14.6846C1.11494 14.4826 1 14.2087 1 13.9231V2.07692C1 1.7913 1.11494 1.51739 1.31952 1.31542C1.52411 1.11346 1.80158 1 2.09091 1H9.18182L13 4.76923V13.9231Z"
            stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M9.18181 7.46152L6.99999 5.30768L4.81818 7.46152" stroke="currentColor" stroke-linecap="round"
              stroke-linejoin="round"/>
        <path d="M7 5.30768V11.2308" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>,
    [IconName.Variation]: <svg width="14" height="14" viewBox="0 0 14 14" fill="none"
                               xmlns="http://www.w3.org/2000/svg">
        <g clip-path="url(#clip0_157_102)">
            <path d="M5.5 1H0.5V6H5.5V1Z" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M13.25 13.5H8.25" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M8.25 8.5H13.25" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M8.25 11H13.25" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M13.5 6H8L10.75 0.5L13.5 6Z" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
            <path
                d="M3 13.5C4.38071 13.5 5.5 12.3807 5.5 11C5.5 9.61929 4.38071 8.5 3 8.5C1.61929 8.5 0.5 9.61929 0.5 11C0.5 12.3807 1.61929 13.5 3 13.5Z"
                stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>
        </g>
        <defs>
            <clipPath id="clip0_157_102">
                <rect width="14" height="14" fill="white"/>
            </clipPath>
        </defs>
    </svg>
}

export default function Icon(
    {
        name,
        width = 32,
        height = 32
    }: Props
): React.JSX.Element {
    return iconMap[name]

}

//===================================================================
// Exports 
//===================================================================